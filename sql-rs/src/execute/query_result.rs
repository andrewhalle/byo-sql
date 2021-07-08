use std::fmt::{Display, Formatter};

use super::{CreateTableQueryResult, InsertQueryResult, SelectQueryResult};

/// TODO short description
pub enum QueryResult {
    SelectQueryResult(SelectQueryResult),
    InsertQueryResult(InsertQueryResult),
    CreateTableQueryResult(CreateTableQueryResult),
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
