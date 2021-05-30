use std::collections::HashSet;
use std::fmt::{Display, Formatter};
use std::io::{self, BufRead, Write};
use std::mem;

#[macro_use]
extern crate pest_derive;
use pest::error::Error;
use pest::iterators::Pair;
use pest::Parser;

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

#[derive(Clone)]
struct Column {
    name: String,
    datatype: Datatype,
}

// TODO this and Value are very similar
#[derive(PartialEq, Clone, Copy)]
enum Datatype {
    Number,
    Text,
}

// TODO this and Datatype are very similar
#[derive(Debug, Clone, PartialEq)]
enum Value {
    Null,
    Number(u32),
    Text(String),
}

impl Value {
    fn assignable_to(&self, datatype: Datatype) -> bool {
        match self {
            Value::Null => true,
            Value::Number(_) => datatype == Datatype::Number,
            Value::Text(_) => datatype == Datatype::Text,
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
            _ => unreachable!(),
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Value::Number(n) => write!(f, "{}", n),
            Value::Text(s) => write!(f, "{}", s),
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
}

#[derive(Debug)]
struct SelectQuery<'a> {
    select_list: Vec<&'a str>,
    table: &'a str,
    filter: Option<Filter<'a>>,
}

#[derive(Debug)]
enum Filter<'a> {
    Eq(FilterEq<'a>),
}

#[derive(Debug)]
struct FilterEq<'a> {
    column: &'a str,
    value: Value,
}

impl<'a> SelectQuery<'a> {
    fn from(select_query: Pair<'a, Rule>) -> Self {
        let mut inner = select_query.into_inner();

        let select_list = inner.next().unwrap();
        let select_list = {
            let inner: Vec<_> = select_list.into_inner().collect();
            if inner.len() == 0 {
                vec!["*"]
            } else {
                inner.iter().map(|identifier| identifier.as_str()).collect()
            }
        };

        let table = inner.next().unwrap().as_str();

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

                        let column = inner.next().unwrap().as_str();
                        let value = Value::from(inner.next().unwrap());

                        retval.filter = Some(Filter::Eq(FilterEq { column, value }));
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

impl<'a> Query<'a> {
    fn from(source: &'a str) -> Result<Self, Error<Rule>> {
        let mut parse = QueryParser::parse(Rule::query, source)?;

        let query = parse.next().unwrap();
        let query = query.into_inner().next().unwrap();
        let query = match query.as_rule() {
            Rule::select_query => Query::SelectQuery(SelectQuery::from(query)),
            Rule::insert_query => Query::InsertQuery(InsertQuery::from(query)),
            _ => unreachable!(),
        };

        Ok(query)
    }
}

enum QueryResult {
    SelectQueryResult(SelectQueryResult),
    InsertQueryResult(InsertQueryResult),
}

struct SelectQueryResult {
    columns: Vec<Column>,
    rows: Vec<Row>,
}

struct InsertQueryResult {
    num_inserted: u32,
}

impl SelectQueryResult {
    fn select(&mut self, columns: Vec<&str>) {
        if columns.len() == 1 && columns[0] == "*" {
            return;
        } else {
            let indices: Vec<usize> = columns
                .iter()
                .map(|c| {
                    self.columns
                        .iter()
                        .map(|c| &c.name)
                        .enumerate()
                        .find(|(_, x)| x == c)
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

    fn filter(&mut self, filter: &Filter<'_>) {
        match filter {
            Filter::Eq(filter) => {
                let idx = self
                    .columns
                    .iter()
                    .enumerate()
                    .find(|(_, c)| c.name == filter.column)
                    .unwrap()
                    .0;

                let predicate = |row: &Row| row.0[idx] == filter.value;

                let mut rows = Vec::new();
                mem::swap(&mut rows, &mut self.rows);

                for row in rows.into_iter() {
                    if predicate(&row) {
                        self.rows.push(row);
                    }
                }
            }
        }
    }
}

impl Display for SelectQueryResult {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        let column_names: Vec<&str> = self.columns.iter().map(|c| c.name.as_str()).collect();
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

impl Display for QueryResult {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            QueryResult::SelectQueryResult(qr) => write!(f, "{}", qr),
            QueryResult::InsertQueryResult(qr) => write!(f, "{}", qr),
        }
    }
}

impl Database {
    fn new() -> Self {
        Database { tables: Vec::new() }
    }

    // TODO make this generic over a Rust struct implementing something like a IntoTable trait
    // currently adds a fixed user table with a fixed row
    fn add_table(&mut self) {
        let user1 = vec![
            Value::Text("1".to_string()),
            Value::Text("user1@email.com".to_string()),
            Value::Number(25),
        ];
        let user2 = vec![
            Value::Text("2".to_string()),
            Value::Text("user2@email.com".to_string()),
            Value::Number(27),
        ];

        self.tables.push(Table {
            name: "users".to_string(),
            rows: vec![Row(user1), Row(user2)],
            columns: vec![
                Column {
                    name: "id".to_string(),
                    datatype: Datatype::Text,
                },
                Column {
                    name: "email".to_string(),
                    datatype: Datatype::Text,
                },
                Column {
                    name: "age".to_string(),
                    datatype: Datatype::Number,
                },
            ],
        });
    }

    fn execute(&mut self, query: Query) -> QueryResult {
        match query {
            Query::SelectQuery(query) => {
                let table = &self.tables[0];
                let mut result = SelectQueryResult {
                    columns: table.columns.clone(),
                    rows: table.rows.clone(),
                };

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

                let table = &mut self.tables[0];
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
}

#[derive(Debug)]
struct User {
    id: String,
    email: String,
    age: u32,
}

fn main() {
    let mut db = Database::new();
    db.add_table();
    db.console();
}
