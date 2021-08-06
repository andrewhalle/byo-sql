use std::fmt::{Display, Formatter};

use crate::data::{Table, Value};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ValueList(Vec<Value>);

impl Display for ValueList {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "(")?;
        let num_values = self.0.len();
        if num_values > 0 {
            for value in &self.0[..num_values - 1] {
                write!(f, "{},", value)?;
            }
            write!(f, "{}", &self.0[num_values - 1])?;
        }
        write!(f, ")")
    }
}

impl From<Table> for ValueList {
    fn from(mut table: Table) -> ValueList {
        assert_eq!(table.columns.len(), 1);

        ValueList(
            table
                .rows
                .iter_mut()
                .map(|r| &mut r.0)
                .map(Vec::pop)
                .map(Option::unwrap)
                .collect(),
        )
    }
}

impl ValueList {
    pub fn contains(&self, other: Value) -> bool {
        self.0.as_slice().contains(&other)
    }
}
