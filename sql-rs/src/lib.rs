//! A crate for parsing and executing SQL in-memory against a simple database representation.

use std::cmp::{Ordering, Reverse};
use std::collections::{HashMap, HashSet};
use std::fmt::{Display, Formatter};
use std::mem;

#[macro_use]
extern crate pest_derive;

/// Data representation.
pub mod data;
use data::{Column, Datatype, Row, Value};
/// Executing a query against a database.
pub mod execute;
/// Parsing SQL.
pub mod parse;
use parse::ast::{self, *};

struct Table {
    name: String,
    columns: Vec<Column>,
    rows: Vec<Row>,
}

fn new_alias_map(table: &TableIdentifier<'_>) -> HashMap<String, String> {
    let mut map = HashMap::new();
    match &table.alias {
        None => map.insert(table.name.0.to_owned(), table.name.0.to_owned()),
        Some(alias) => map.insert(alias.0.to_owned(), table.name.0.to_owned()),
    };

    map
}

fn cmp_column_with_column_identifier(
    c1: &SelectQueryResultColumn,
    c2: &ColumnIdentifier<'_>,
    table_alias_map: &HashMap<String, String>,
) -> bool {
    let is_star = matches!(c2.name, ast::Column::Star);
    let c2_column = match &c2.name {
        ast::Column::Star => None,
        ast::Column::Ident(i) => Some(i.0),
    };
    let names_match = is_star || c1.column == c2_column.unwrap();

    match c2.alias.as_ref() {
        None => names_match,
        Some(alias) => &c1.table == table_alias_map.get(alias.0).unwrap() && names_match,
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

#[derive(PartialEq, Eq)]
struct SortableValue(Value);

// TODO can this implementation be derived?
impl Ord for SortableValue {
    fn cmp(&self, other: &Self) -> Ordering {
        match &self.0 {
            Value::Null => match &other.0 {
                Value::Null => Ordering::Equal,
                _ => Ordering::Less,
            },
            Value::Number(n1) => match &other.0 {
                Value::Null => Ordering::Greater,
                Value::Number(n2) => n1.cmp(&n2),
                _ => Ordering::Less,
            },
            Value::Text(s1) => match &other.0 {
                Value::Null | Value::Number(_) => Ordering::Greater,
                Value::Text(s2) => s1.cmp(&s2),
                _ => Ordering::Less,
            },
            Value::Boolean(b1) => match &other.0 {
                Value::Null | Value::Number(_) | Value::Text(_) => Ordering::Greater,
                Value::Boolean(b2) => b1.cmp(&b2),
            },
        }
    }
}

impl PartialOrd for SortableValue {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
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

    // TODO impl From<Literal<'_>>
    fn from(literal: Literal<'_>) -> Self {
        todo!()
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

impl<'input> Expression<'input> {
    // TODO make better / re-locate
    fn is_column_star(&self) -> bool {
        matches!(
            self,
            ast::Expression::ColumnIdentifier(ColumnIdentifier {
                name: ast::Column::Star,
                ..
            })
        )
    }

    // TODO make better / re-locate
    fn is_count(&self) -> bool {
        matches!(self, ast::Expression::CountStar)
    }
}

enum QueryResult {
    SelectQueryResult(SelectQueryResult),
    InsertQueryResult(InsertQueryResult),
    CreateTableQueryResult(CreateTableQueryResult),
}

#[derive(Debug, Clone)]
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
            datatype: column.datatype,
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

#[derive(PartialEq, Eq, PartialOrd, Ord)]
enum OrdVariants {
    SortableValue(SortableValue),
    Reversed(Reverse<SortableValue>),
}

impl SelectQueryResult {
    // TODO should this clone?
    fn get_column_value_from_row(
        &self,
        column_identifier: &ColumnIdentifier<'_>,
        row: &Row,
    ) -> Value {
        assert!(!matches!(column_identifier.name, ast::Column::Star));

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

    // TODO re-write using ast::Expression
    fn evaluate(&self, expr: &Expression<'_>, row: &Row) -> Value {
        match expr {
            ast::Expression::ColumnIdentifier(i) => self.get_column_value_from_row(i, row),
            ast::Expression::Literal(l) => l.into(),
            ast::Expression::BinaryOp(b) => {
                let v1 = self.evaluate(&b.left, row);
                let v2 = self.evaluate(&b.right, row);

                v1.op(b.op, v2)
            }
            _ => unreachable!(),
        }
    }

    fn select(&mut self, select_list: Vec<Expression<'_>>) -> Self {
        let mut retval = SelectQueryResult {
            columns: Vec::new(),
            rows: Vec::new(),
            table_alias_map: HashMap::new(),
        };

        // TODO functions likely needed
        //   * get column description from expression
        //   * get type of expression
        // push columns
        for expr in &select_list {
            if expr.is_count() {
                retval.columns.push(SelectQueryResultColumn {
                    table: "".to_string(),
                    column: "count".to_string(),
                    datatype: Datatype::Number,
                });
                break;
            }

            let column_identifier = match expr {
                ast::Expression::ColumnIdentifier(i) => i,
                _ => unreachable!(),
            };
            if matches!(column_identifier.name, ast::Column::Star) {
                for column in &self.columns {
                    if cmp_column_with_column_identifier(
                        column,
                        &column_identifier,
                        &self.table_alias_map,
                    ) {
                        retval.columns.push(column.clone());
                    }
                }
            } else {
                let column = self
                    .columns
                    .iter()
                    .find(|x| {
                        cmp_column_with_column_identifier(
                            x,
                            &column_identifier,
                            &self.table_alias_map,
                        )
                    })
                    .unwrap();
                retval.columns.push(column.clone());
            }
        }

        // push projected rows
        if select_list[0].is_count() {
            let mut row = Vec::new();
            row.push(Value::Number(self.rows.len() as u32));
            retval.rows.push(Row(row));
        } else {
            for row in &self.rows {
                let mut new_row = Vec::new();
                for expr in &select_list {
                    if expr.is_column_star() {
                        let column_identifier = match expr {
                            ast::Expression::ColumnIdentifier(i) => i,
                            _ => unreachable!(),
                        };
                        for (i, column) in self.columns.iter().enumerate() {
                            if cmp_column_with_column_identifier(
                                column,
                                &column_identifier,
                                &self.table_alias_map,
                            ) {
                                new_row.push(row.0[i].clone());
                            }
                        }
                    } else {
                        new_row.push(self.evaluate(&expr, row));
                    }
                }
                retval.rows.push(Row(new_row));
            }
        }

        retval
    }

    // TODO move this to some sort of TableView once it exists.
    /// Filters a SelectQueryResult by evaluating expression for each row, and keeping it if the
    /// expression evaluates to true.
    fn filter(&mut self, expression: &Expression<'_>) {
        let mut rows = Vec::new();
        mem::swap(&mut rows, &mut self.rows);

        for row in rows.into_iter() {
            if self.evaluate(&expression, &row).is_true() {
                self.rows.push(row);
            }
        }
    }

    // TODO move this to some sort of TableView once it exists.
    /// Sorts the rows in a SelectQueryResult by evaluating expression and using it as a key.
    fn sort(&mut self, order_by: &OrderBy<'_>) {
        let mut rows = mem::take(&mut self.rows);

        rows.as_mut_slice().sort_unstable_by_key(|row| {
            let sortable = SortableValue(self.evaluate(&order_by.expr, &row));

            match order_by.direction {
                OrderByDirection::Asc => OrdVariants::SortableValue(sortable),
                OrderByDirection::Desc => OrdVariants::Reversed(Reverse(sortable)),
            }
        });

        mem::swap(&mut rows, &mut self.rows);
    }

    fn limit(&mut self, rows: u32) {
        self.rows.truncate(rows as usize);
    }

    // TODO nested loop join, considering join.kind. Probably need to re-write self.evaluate
    // so that it can work on a row that's not in self yet (since we need to check if we want to
    // add it).
    fn join(&mut self, rhs: &mut Self, join: &Join<'_>) {
        let lhs_column_count = self.columns.len();
        let rhs_column_count = rhs.columns.len();

        self.columns.append(&mut rhs.columns);
        for (k, v) in rhs.table_alias_map.drain() {
            self.table_alias_map.insert(k, v);
        }

        let outer_iter = || match join.kind {
            JoinKind::Right => rhs.rows.iter(),
            _ => self.rows.iter(),
        };
        let inner_iter = || match join.kind {
            JoinKind::Right => self.rows.iter(),
            _ => rhs.rows.iter(),
        };

        let mut rows = Vec::new();
        for i in outer_iter() {
            let mut did_add_row = false;

            for j in inner_iter() {
                let mut row = i.0.clone();
                row.append(&mut j.0.clone());
                let row = Row(row);
                if self.evaluate(&join.condition, &row).is_true() {
                    rows.push(row);
                    did_add_row = true;
                }
            }

            if !did_add_row {
                let row = match join.kind {
                    JoinKind::Left => {
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
                    JoinKind::Right => {
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
