use std::fmt::{Display, Formatter};

use super::Datatype;

/// TODO short description.
///
/// TODO long description.
#[derive(Clone, Debug)]
pub struct Column {
    pub name: String,
    pub datatype: Datatype,
}

impl Display for Column {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        match self.name.find(".") {
            None => write!(f, "{}", self.name),
            Some(idx) => {
                let (_, name) = self.name.split_at(idx + 1);

                write!(f, "{}", name)
            }
        }
    }
}
