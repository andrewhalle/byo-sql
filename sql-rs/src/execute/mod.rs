///! TODO long module description.

/// TODO doc
mod query_result;
pub use query_result::*;

/// TODO doc
mod select_query_result;
pub use select_query_result::*;

/// TODO doc
mod insert;
pub use insert::*;

/// TODO doc
mod create_table;
pub use create_table::*;

use std::fmt::{Display, Formatter};

use crate::data::Database;
use crate::parse::ast::Query;

#[derive(Debug)]
pub enum Success {
    CreateTable(create_table::Success),
    Insert(insert::Success),
    // TODO remove me
    OldImplementation(QueryResult),
}

impl Display for Success {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        match self {
            Success::CreateTable(s) => write!(f, "{}", s),
            Success::Insert(s) => write!(f, "{}", s),
            Success::OldImplementation(s) => write!(f, "{}", s),
        }
    }
}

#[derive(Debug)]
pub enum Error {
    QueryFailed,
}

impl Display for Error {
    fn fmt(&self, _f: &mut Formatter) -> Result<(), std::fmt::Error> {
        todo!()
    }
}

impl From<create_table::Error> for Error {
    fn from(_: create_table::Error) -> Self {
        Error::QueryFailed
    }
}

impl From<insert::Error> for Error {
    fn from(_: insert::Error) -> Self {
        Error::QueryFailed
    }
}

impl Database {
    pub fn execute(&mut self, query: Query<'_>) -> Result<Success, Error> {
        Ok(match query {
            Query::InsertQuery(query) => Success::Insert(self.execute_insert(query)?),
            Query::CreateTableQuery(query) => {
                Success::CreateTable(self.execute_create_table(query)?)
            }
            _ => Success::OldImplementation(self.execute_old(query)),
        })
    }
}
