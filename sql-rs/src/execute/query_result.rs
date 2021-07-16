use std::fmt::{Display, Formatter};

use super::SelectQueryResult;

#[derive(Debug)]
/// TODO short description
pub enum QueryResult {
    SelectQueryResult(SelectQueryResult),
}

impl Display for QueryResult {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            QueryResult::SelectQueryResult(qr) => write!(f, "{}", qr),
        }
    }
}
