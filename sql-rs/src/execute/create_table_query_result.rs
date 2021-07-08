use std::fmt::{Display, Formatter};

pub struct CreateTableQueryResult;

impl Display for CreateTableQueryResult {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "CREATED TABLE")
    }
}
