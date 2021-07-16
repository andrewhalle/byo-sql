use std::fmt::{Display, Formatter};

use crate::data::{Database, Row};
use crate::parse::ast::InsertQuery;

#[derive(Debug)]
pub struct Success {
    num_inserted: u32,
}

impl Display for Success {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "INSERT {}", self.num_inserted)
    }
}

#[derive(Debug)]
pub enum Error {
    IncorrectColumnNumber,
    IncompatibleDatatype,
}

impl Display for Error {
    fn fmt(&self, _f: &mut Formatter) -> Result<(), std::fmt::Error> {
        todo!()
    }
}

type QueryResult = Result<Success, Error>;

impl Database {
    pub fn execute_insert(&mut self, query: InsertQuery<'_>) -> QueryResult {
        if query.columns.len() != query.values.len() {
            return Err(Error::IncorrectColumnNumber);
        }

        let table = self.find_table_mut(query.table.0);
        let mut indices = table
            .validate_insert_query_columns(
                &(query.columns.iter().map(|i| i.0).collect::<Vec<&str>>()),
            )
            .unwrap();
        let mut row = table.new_values_vec();
        let mut values = query.values;

        while values.len() != 0 {
            let i = indices.pop().unwrap();
            let value = values.pop().unwrap();

            let value = (&value).into();
            if !table.compatible_type(i, &value) {
                return Err(Error::IncompatibleDatatype);
            }

            row[i] = value;
        }

        table.rows.push(Row(row));

        Ok(Success { num_inserted: 1 })
    }
}
