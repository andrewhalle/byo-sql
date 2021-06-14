use std::collections::{HashMap, HashSet};
use std::fmt::{Display, Formatter};
use std::fs;
use std::io::{self, BufRead, Write};
use std::mem;
use std::path::PathBuf;
use std::process;

#[macro_use]
extern crate pest_derive;
use pest::error::Error;
use pest::iterators::Pair;
use pest::prec_climber::{Operator, PrecClimber};
use pest::Parser;

use structopt::StructOpt;

use ansi_term::Colour;

use lazy_static::lazy_static;

lazy_static! {
    static ref PREC_CLIMBER: PrecClimber<Rule> = {
        use pest::prec_climber::Assoc::*;
        use Rule::*;

        PrecClimber::new(vec![
            Operator::new(and, Left) | Operator::new(or, Left),
            Operator::new(greater_equal, Left)
                | Operator::new(greater, Left)
                | Operator::new(less_equal, Left)
                | Operator::new(less, Left)
                | Operator::new(equal, Left),
        ])
    };
}

#[derive(Parser)]
#[grammar = "query.pest"]
pub struct QueryParser;

struct Database {
    tables: Vec<Table>,
}

struct Table {
    name: String,
    columns: Vec<Column>,
    rows: Vec<Row>,
}

#[derive(Debug)]
struct TableIdentifier<'a> {
    name: &'a str,
    alias: Option<&'a str>,
}

fn new_alias_map(table: &TableIdentifier<'_>) -> HashMap<String, String> {
    let mut map = HashMap::new();
    match table.alias {
        None => map.insert(table.name.to_owned(), table.name.to_owned()),
        Some(alias) => map.insert(alias.to_owned(), table.name.to_owned()),
    };

    map
}

fn cmp_column_with_column_identifier(
    c1: &SelectQueryResultColumn,
    c2: &ColumnIdentifier<'_>,
    table_alias_map: &HashMap<String, String>,
) -> bool {
    match c2.alias {
        None => c1.column == c2.column,
        Some(alias) => &c1.table == table_alias_map.get(alias).unwrap() && c1.column == c2.column,
    }
}

// TODO document why this makes sense to handle "*" as a column name, even if no table alias is
// provided (e.g. mysql vs postgres on "select *, * from test;")
#[derive(Debug)]
struct ColumnIdentifier<'a> {
    alias: Option<&'a str>,
    column: &'a str,
}

impl<'a> TableIdentifier<'a> {
    fn from(table_identifier: Pair<'a, Rule>) -> Self {
        assert_eq!(table_identifier.as_rule(), Rule::table_identifier);

        let mut inner = table_identifier.into_inner();

        let name = inner.next().unwrap().as_str();
        let alias = inner.next().map(|p| p.as_str());

        TableIdentifier { name, alias }
    }
}

impl<'a> ColumnIdentifier<'a> {
    fn from(column_identifier: Pair<'a, Rule>) -> Self {
        assert_eq!(column_identifier.as_rule(), Rule::column_identifier);

        let mut pairs: Vec<Pair<'_, Rule>> = column_identifier.into_inner().collect();

        let (alias, column) = if pairs.len() == 1 {
            (None, pairs.pop().unwrap().as_str())
        } else {
            let column = pairs.pop().unwrap().as_str();
            (Some(pairs.pop().unwrap().as_str()), column)
        };

        ColumnIdentifier { alias, column }
    }
}

impl Table {
    fn validate_insert_query_columns(&self, insert_query_columns: &[&str]) -> Option<Vec<usize>> {
        // first check that all columns are provided
        {
            let self_columns: HashSet<&str> =
                self.columns.iter().map(|c| c.name.as_str()).collect();
            let insert_query_columns: HashSet<&str> =
                insert_query_columns.iter().map(|c| *c).collect();

            if self_columns != insert_query_columns {
                return None;
            }
        }

        // then map the insert_query_column to the index in the table specificiation
        let self_columns: Vec<(usize, &str)> = self
            .columns
            .iter()
            .map(|c| c.name.as_str())
            .enumerate()
            .collect();
        let indices = insert_query_columns
            .iter()
            .map(|c1| self_columns.iter().find(|(_, c2)| c1 == c2).unwrap().0)
            .collect();
        Some(indices)
    }

    fn new_values_vec(&self) -> Vec<Value> {
        vec![Value::Null; self.columns.len()]
    }

    fn compatible_type(&self, column_index: usize, value: &Value) -> bool {
        let column = &self.columns[column_index];
        value.assignable_to(column.datatype)
    }
}

#[derive(Clone, Debug)]
struct Column {
    name: String,
    datatype: Datatype,
}

// TODO this and Value are very similar
#[derive(PartialEq, Clone, Copy, Debug)]
enum Datatype {
    Number,
    Text,
    Boolean,
}

impl Datatype {
    fn from_pair(pair: Pair<'_, Rule>) -> Self {
        assert_eq!(pair.as_rule(), Rule::datatype);

        match pair.as_str() {
            "number" => Datatype::Number,
            "text" => Datatype::Text,
            "boolean" => Datatype::Boolean,
            _ => unreachable!(),
        }
    }
}

// TODO this and Datatype are very similar
#[derive(Debug, Clone, PartialEq)]
enum Value {
    Null,
    Number(u32),
    Text(String),
    Boolean(bool),
}

macro_rules! value_op {
    ($name:ident, $op:tt) => {
        impl Value {
            fn $name(&self, rhs: &Value) -> bool {
                match self {
                    Value::Text(_) => self.as_str() $op rhs.as_str(),
                    Value::Number(_) => self.as_number() $op rhs.as_number(),
                    _ => panic!("operands of $op must be text or number"),
                }
            }
        }
    };
}

impl Value {
    fn assignable_to(&self, datatype: Datatype) -> bool {
        match self {
            Value::Null => true,
            Value::Number(_) => datatype == Datatype::Number,
            Value::Text(_) => datatype == Datatype::Text,
            Value::Boolean(_) => datatype == Datatype::Boolean,
        }
    }

    fn from(literal: Pair<'_, Rule>) -> Self {
        assert_eq!(literal.as_rule(), Rule::literal);

        let literal = literal.into_inner().next().unwrap();
        match literal.as_rule() {
            Rule::string_literal => {
                let string_literal_contents = literal.into_inner().next().unwrap();

                Value::Text(string_literal_contents.as_str().to_owned())
            }
            Rule::number_literal => {
                let num: u32 = literal.as_str().parse().unwrap();

                Value::Number(num)
            }
            Rule::boolean_literal => match literal.as_str() {
                "true" => Value::Boolean(true),
                "false" => Value::Boolean(false),
                _ => unreachable!(),
            },
            _ => unreachable!(),
        }
    }

    fn is_true(&self) -> bool {
        match self {
            Value::Boolean(b) => *b,
            _ => panic!("cannot use a non-boolean Value in a boolean context"),
        }
    }

    fn as_number(&self) -> u32 {
        match self {
            Value::Number(n) => *n,
            _ => panic!(),
        }
    }

    fn as_str(&self) -> &str {
        match self {
            Value::Text(s) => s.as_str(),
            _ => panic!(),
        }
    }
}
value_op!(greater_equal, >=);
value_op!(greater, >);
value_op!(less_equal, <=);
value_op!(less, <);

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Value::Number(n) => write!(f, "{}", n),
            Value::Text(s) => write!(f, "{}", s),
            Value::Boolean(b) => write!(f, "{}", b),
            Value::Null => write!(f, "null"),
        }
    }
}

#[derive(Clone)]
struct Row(Vec<Value>);

#[derive(Debug)]
enum Query<'a> {
    SelectQuery(SelectQuery<'a>),
    InsertQuery(InsertQuery<'a>),
    CreateTableQuery(CreateTableQuery<'a>),
}

#[derive(Debug)]
struct SelectQuery<'a> {
    select_list: Vec<ColumnIdentifier<'a>>,
    table: TableSelection<'a>,
    filter: Option<Expression<'a>>,
}

#[derive(Debug)]
struct Expression<'a> {
    inner: Pair<'a, Rule>,
}

#[derive(Debug)]
struct TableSelection<'a> {
    root_table: TableIdentifier<'a>,
    joins: Vec<TableJoin<'a>>,
}

#[derive(Debug)]
struct TableJoin<'a> {
    join_type: JoinType,
    table: TableIdentifier<'a>,
    expr: Expression<'a>,
}

#[derive(Debug)]
enum JoinType {
    Inner,
    Left,
    Right,
}

impl JoinType {
    fn from(join_type: Pair<'_, Rule>) -> Self {
        assert_eq!(join_type.as_rule(), Rule::join_type);
        let join_type = join_type.into_inner().next().unwrap();
        match join_type.as_rule() {
            Rule::inner_join => JoinType::Inner,
            Rule::left_join => JoinType::Left,
            Rule::right_join => JoinType::Right,
            _ => unreachable!(),
        }
    }
}

impl<'a> TableJoin<'a> {
    fn from(join: Pair<'a, Rule>) -> Self {
        assert_eq!(join.as_rule(), Rule::join_clause);

        let mut inner = join.into_inner();

        let join_type = JoinType::from(inner.next().unwrap());
        let table = TableIdentifier::from(inner.next().unwrap());
        let expr = Expression {
            inner: inner.next().unwrap(),
        };

        TableJoin {
            join_type,
            table,
            expr,
        }
    }
}

impl<'a> SelectQuery<'a> {
    fn from(select_query: Pair<'a, Rule>) -> Self {
        let mut inner = select_query.into_inner();

        let select_list = inner.next().unwrap();
        let select_list = {
            let inner: Vec<_> = select_list.into_inner().collect();
            inner.into_iter().map(ColumnIdentifier::from).collect()
        };

        let table = {
            let table_expression = inner.next().unwrap();
            let mut inner = table_expression.into_inner();

            let root_table = TableIdentifier::from(inner.next().unwrap());

            let mut joins = Vec::new();
            // remaining tokens are joins
            for join in inner {
                joins.push(TableJoin::from(join));
            }

            TableSelection { root_table, joins }
        };

        let mut retval = SelectQuery {
            select_list,
            table,
            filter: None,
        };

        loop {
            let optional = inner.next();
            if let Some(tree) = optional {
                match tree.as_rule() {
                    Rule::where_clause => {
                        let mut inner = tree.into_inner();
                        let expression = inner.next().unwrap();

                        retval.filter = Some(Expression { inner: expression });
                    }
                    _ => unreachable!(),
                }
            } else {
                break;
            }
        }

        retval
    }
}

#[derive(Debug)]
struct InsertQuery<'a> {
    table: &'a str,
    column_list: Vec<&'a str>,
    values: Vec<Value>,
}

impl<'a> InsertQuery<'a> {
    fn from(insert_query: Pair<'a, Rule>) -> Self {
        let mut inner = insert_query.into_inner();

        let table = inner.next().unwrap().as_str();

        let column_list = {
            let inner = inner.next().unwrap().into_inner();
            let mut column_list = Vec::new();

            for identifier in inner {
                column_list.push(identifier.as_str());
            }

            column_list
        };

        let values = {
            let inner = inner.next().unwrap().into_inner();
            let mut values = Vec::new();

            for literal in inner {
                let value = Value::from(literal);

                values.push(value);
            }

            values
        };

        InsertQuery {
            table,
            column_list,
            values,
        }
    }
}

#[derive(Debug)]
struct CreateTableQuery<'a> {
    table_name: &'a str,
    columns: Vec<Column>,
}

impl<'a> CreateTableQuery<'a> {
    fn from(create_table_query: Pair<'a, Rule>) -> Self {
        let mut inner = create_table_query.into_inner();

        let table_name = inner.next().unwrap().as_str();

        let columns = {
            let create_table_column_list = inner.next().unwrap();
            let mut columns = Vec::new();
            for create_table_column in create_table_column_list.into_inner() {
                let mut inner = create_table_column.into_inner();
                let name = inner.next().unwrap().as_str().to_owned();
                let datatype = Datatype::from_pair(inner.next().unwrap());
                columns.push(Column { name, datatype });
            }

            columns
        };

        CreateTableQuery {
            table_name,
            columns,
        }
    }
}

impl<'a> Query<'a> {
    fn from(source: &'a str) -> Result<Self, Error<Rule>> {
        let mut parse = QueryParser::parse(Rule::query, source)?;

        let query = parse.next().unwrap();
        let query = query.into_inner().next().unwrap();
        let query = match query.as_rule() {
            Rule::select_query => Query::SelectQuery(SelectQuery::from(query)),
            Rule::insert_query => Query::InsertQuery(InsertQuery::from(query)),
            Rule::create_table_query => Query::CreateTableQuery(CreateTableQuery::from(query)),
            _ => unreachable!(),
        };

        Ok(query)
    }

    // TODO remove duplication
    fn get_many(source: &'a str) -> Result<Vec<Self>, Error<Rule>> {
        let mut parse = QueryParser::parse(Rule::queries, source)?;
        let queries = parse.next().unwrap();
        let inner = queries.into_inner();

        let mut queries = Vec::new();
        for query in inner {
            let query = query.into_inner().next().unwrap();
            let query = match query.as_rule() {
                Rule::select_query => Query::SelectQuery(SelectQuery::from(query)),
                Rule::insert_query => Query::InsertQuery(InsertQuery::from(query)),
                Rule::create_table_query => Query::CreateTableQuery(CreateTableQuery::from(query)),
                _ => unreachable!(),
            };
            queries.push(query);
        }

        Ok(queries)
    }
}

enum QueryResult {
    SelectQueryResult(SelectQueryResult),
    InsertQueryResult(InsertQueryResult),
    CreateTableQueryResult(CreateTableQueryResult),
}

#[derive(Clone)]
struct SelectQueryResultColumn {
    table: String,
    column: String,
    datatype: Datatype,
}

impl SelectQueryResultColumn {
    fn from(table: String, column: &Column) -> Self {
        SelectQueryResultColumn {
            table,
            column: column.name.clone(),
            datatype: column.datatype.clone(),
        }
    }
}

struct SelectQueryResult {
    columns: Vec<SelectQueryResultColumn>,
    rows: Vec<Row>,
    table_alias_map: HashMap<String, String>,
}

struct InsertQueryResult {
    num_inserted: u32,
}

struct CreateTableQueryResult;

impl SelectQueryResult {
    // TODO should this clone?
    fn get_column_value_from_row(
        &self,
        column_identifier: ColumnIdentifier<'_>,
        row: &Row,
    ) -> Value {
        let idx = self
            .columns
            .iter()
            .enumerate()
            .find(|(_, x)| {
                cmp_column_with_column_identifier(x, &column_identifier, &self.table_alias_map)
            })
            .unwrap()
            .0;

        row.0[idx].clone()
    }

    fn evaluate(&self, expr: Expression<'_>, row: &Row) -> Value {
        PREC_CLIMBER.climb(
            expr.inner.into_inner(),
            |pair: Pair<Rule>| match pair.as_rule() {
                Rule::column_identifier => {
                    self.get_column_value_from_row(ColumnIdentifier::from(pair), row)
                }
                Rule::expression => self.evaluate(
                    Expression {
                        inner: pair.clone(),
                    },
                    row,
                ),
                Rule::literal => Value::from(pair),
                _ => unreachable!(),
            },
            |lhs: Value, op: Pair<Rule>, rhs: Value| match op.as_rule() {
                Rule::greater_equal => Value::Boolean(lhs.greater_equal(&rhs)),
                Rule::less_equal => Value::Boolean(lhs.less_equal(&rhs)),
                Rule::greater => Value::Boolean(lhs.greater(&rhs)),
                Rule::less => Value::Boolean(lhs.less(&rhs)),
                Rule::and => Value::Boolean(lhs.is_true() && rhs.is_true()),
                Rule::or => Value::Boolean(lhs.is_true() || rhs.is_true()),
                Rule::equal => Value::Boolean(lhs == rhs),
                _ => unreachable!(),
            },
        )
    }

    fn select(&mut self, columns: Vec<ColumnIdentifier<'_>>) {
        // TODO make this work with multiple "*" (e.g. "select *, * from test;"), probably get rid
        // of the length check altogether
        if columns.len() == 1 && columns[0].column == "*" {
            return;
        } else {
            let indices: Vec<usize> = columns
                .iter()
                .map(|c| {
                    self.columns
                        .iter()
                        .enumerate()
                        .find(|(_, x)| {
                            cmp_column_with_column_identifier(x, c, &self.table_alias_map)
                        })
                        .unwrap()
                        .0
                })
                .collect();

            let mut new_columns = Vec::new();
            for i in indices.iter() {
                new_columns.push(self.columns[*i].clone());
            }
            self.columns = new_columns;

            for row in self.rows.iter_mut() {
                let mut new_row = Vec::new();
                for i in indices.iter() {
                    new_row.push(row.0[*i].clone());
                }
                *row = Row(new_row);
            }

            return;
        }
    }

    // TODO move this to some sort of TableView once it exists.
    /// Filters a SelectQueryResult by evaluating expression for each row, and keeping it if the
    /// expression evaluates to true.
    fn filter(&mut self, expression: &Expression<'_>) {
        // clone the expression
        let mut rows = Vec::new();
        mem::swap(&mut rows, &mut self.rows);

        for row in rows.into_iter() {
            if self
                .evaluate(
                    Expression {
                        inner: expression.inner.clone(),
                    },
                    &row,
                )
                .is_true()
            {
                self.rows.push(row);
            }
        }
    }

    // TODO nested loop join, considering join.join_type. Probably need to re-write self.evaluate
    // so that it can work on a row that's not in self yet (since we need to check if we want to
    // add it).
    fn join(&mut self, rhs: &mut Self, join: &TableJoin) {
        let lhs_column_count = self.columns.len();
        let rhs_column_count = rhs.columns.len();

        self.columns.append(&mut rhs.columns);

        let outer_iter = || match join.join_type {
            JoinType::Right => rhs.rows.iter(),
            _ => self.rows.iter(),
        };
        let inner_iter = || match join.join_type {
            JoinType::Right => self.rows.iter(),
            _ => rhs.rows.iter(),
        };

        let mut rows = Vec::new();
        for i in outer_iter() {
            let mut did_add_row = false;

            for j in inner_iter() {
                let mut row = i.0.clone();
                row.append(&mut j.0.clone());
                let row = Row(row);
                if self
                    .evaluate(
                        Expression {
                            inner: join.expr.inner.clone(),
                        },
                        &row,
                    )
                    .is_true()
                {
                    rows.push(row);
                    did_add_row = true;
                }
            }

            if !did_add_row {
                let row = match join.join_type {
                    JoinType::Left => {
                        let mut row = i.0.clone();
                        let mut nulls = {
                            let mut nulls = Vec::with_capacity(rhs_column_count);
                            for _i in 0..rhs_column_count {
                                nulls.push(Value::Null);
                            }
                            nulls
                        };
                        row.append(&mut nulls);
                        Some(Row(row))
                    }
                    JoinType::Right => {
                        let mut row = i.0.clone();
                        let mut nulls = {
                            let mut nulls = Vec::with_capacity(lhs_column_count);
                            for _i in 0..lhs_column_count {
                                nulls.push(Value::Null);
                            }
                            nulls
                        };
                        nulls.append(&mut row);
                        Some(Row(nulls))
                    }
                    _ => None,
                };

                if row.is_some() {
                    rows.push(row.unwrap());
                }
            }
        }
        mem::swap(&mut self.rows, &mut rows);
        for (k, v) in rhs.table_alias_map.drain() {
            self.table_alias_map.insert(k, v);
        }
    }
}

impl Display for SelectQueryResult {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        let column_names: Vec<&str> = self.columns.iter().map(|c| c.column.as_str()).collect();
        writeln!(f, "{}", &column_names.join(",")).unwrap();

        let num_rows = self.rows.len();
        for (i, row) in self.rows.iter().enumerate() {
            let values: Vec<String> = row.0.iter().map(|v| v.to_string()).collect();
            write!(f, "{}", &values.join(",")).unwrap();
            if i != num_rows - 1 {
                writeln!(f).unwrap();
            }
        }

        Ok(())
    }
}

impl Display for InsertQueryResult {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "INSERT {}", self.num_inserted)
    }
}

impl Display for CreateTableQueryResult {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "CREATED TABLE")
    }
}

impl Display for QueryResult {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            QueryResult::SelectQueryResult(qr) => write!(f, "{}", qr),
            QueryResult::InsertQueryResult(qr) => write!(f, "{}", qr),
            QueryResult::CreateTableQueryResult(qr) => write!(f, "{}", qr),
        }
    }
}

impl Database {
    fn new() -> Self {
        Database { tables: Vec::new() }
    }

    fn find_table(&self, table: &str) -> &Table {
        self.tables.iter().find(|t| t.name == table).unwrap()
    }

    fn find_table_mut(&mut self, table: &str) -> &mut Table {
        self.tables.iter_mut().find(|t| t.name == table).unwrap()
    }

    fn execute(&mut self, query: Query) -> QueryResult {
        match query {
            Query::SelectQuery(query) => {
                let table = self.find_table(query.table.root_table.name);
                let mut result = SelectQueryResult {
                    columns: table
                        .columns
                        .iter()
                        .map(|c| SelectQueryResultColumn::from(table.name.clone(), c))
                        .collect(),
                    rows: table.rows.clone(),
                    table_alias_map: new_alias_map(&query.table.root_table),
                };

                for join in query.table.joins {
                    let table = self.find_table(join.table.name);
                    let mut table = SelectQueryResult {
                        columns: table
                            .columns
                            .iter()
                            .map(|c| SelectQueryResultColumn::from(table.name.clone(), c))
                            .collect(),
                        rows: table.rows.clone(),
                        table_alias_map: new_alias_map(&join.table),
                    };
                    result.join(&mut table, &join);
                }

                if let Some(filter) = &query.filter {
                    result.filter(filter);
                }

                result.select(query.select_list);

                QueryResult::SelectQueryResult(result)
            }
            Query::InsertQuery(query) => {
                if query.column_list.len() != query.values.len() {
                    panic!();
                }

                let table = self.find_table_mut(query.table);
                let mut indices = table
                    .validate_insert_query_columns(&query.column_list)
                    .unwrap();
                let mut row = table.new_values_vec();
                let mut values = query.values;

                while values.len() != 0 {
                    let i = indices.pop().unwrap();
                    let value = values.pop().unwrap();

                    if !table.compatible_type(i, &value) {
                        panic!();
                    }

                    row[i] = value;
                }

                table.rows.push(Row(row));

                QueryResult::InsertQueryResult(InsertQueryResult { num_inserted: 1 })
            }
            Query::CreateTableQuery(query) => {
                // TODO check that table doesn't exist
                let table = Table {
                    name: query.table_name.to_owned(),
                    columns: query.columns,
                    rows: Vec::new(),
                };
                self.tables.push(table);
                QueryResult::CreateTableQueryResult(CreateTableQueryResult)
            }
        }
    }

    fn console(&mut self) {
        let stdin = io::stdin();
        let mut stdout = io::stdout();
        let mut reader = stdin.lock();
        let mut line = String::new();

        loop {
            line.clear();
            print!("> ");
            stdout.flush().unwrap();
            reader.read_line(&mut line).unwrap();

            if line == "" {
                println!();
                break;
            }

            let query = Query::from(&line);

            match query {
                Ok(query) => {
                    let result = self.execute(query);
                    println!("{}", result);
                }
                Err(parse_error) => {
                    println!("{}", parse_error);
                }
            }
        }
    }

    fn seed(&mut self, seed_file: PathBuf) {
        let seed = fs::read_to_string(seed_file).unwrap();
        let queries = Query::get_many(&seed);

        match queries {
            Ok(queries) => {
                for query in queries {
                    self.execute(query);
                }
            }
            Err(parse_error) => {
                println!("{}", parse_error);
                process::exit(1);
            }
        }

        let style = Colour::Fixed(251).italic();
        println!("{}\n\n{}", style.paint("Seeded with:"), style.paint(&seed));
    }
}

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(short, parse(from_os_str))]
    seed_file: Option<PathBuf>,
}

fn main() {
    let opt = Opt::from_args();

    let mut db = Database::new();
    if let Some(seed_file) = opt.seed_file {
        db.seed(seed_file);
    }

    db.console();
}
