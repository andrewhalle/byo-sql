use std::fmt::{Display, Formatter};

use crate::data::{Column, Database, Table};
use crate::parse::ast::CreateTableQuery;

#[derive(Debug)]
pub struct Success();

impl Display for Success {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "CREATED TABLE")
    }
}

#[derive(Debug)]
pub enum Error {
    TableExists,
}

impl Display for Error {
    fn fmt(&self, _f: &mut Formatter) -> Result<(), std::fmt::Error> {
        todo!()
    }
}

type QueryResult = Result<Success, Error>;

impl Database {
    pub fn execute_create_table(&mut self, query: CreateTableQuery<'_>) -> QueryResult {
        if self.tables.contains_key(query.table_name.0) {
            return Err(Error::TableExists);
        }

        let name = query.table_name.0.to_owned();
        let table = Table {
            columns: query
                .columns
                .iter()
                .map(|c| Column {
                    name: c.name.0.to_owned(),
                    datatype: c.datatype,
                })
                .collect(),
            rows: Vec::new(),
        };
        self.tables.insert(name, table);

        Ok(Success())
    }
}
