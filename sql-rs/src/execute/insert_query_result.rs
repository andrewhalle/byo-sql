use std::fmt::{Display, Formatter};

pub struct InsertQueryResult {
    pub num_inserted: u32,
}

impl Display for InsertQueryResult {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "INSERT {}", self.num_inserted)
    }
}
