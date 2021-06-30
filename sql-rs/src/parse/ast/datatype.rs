/// TODO quick description.
///
/// TODO long description.
// TODO move the enum declaration into the data model, and leave the implementation of From<Pair<..>> here
#[derive(Debug)]
pub enum Datatype {
    Number,
    Text,
    Boolean,
}

use crate::parse::Rule;
use pest::iterators::Pair;

impl From<Pair<'_, Rule>> for Datatype {
    fn from(datatype: Pair<'_, Rule>) -> Self {
        assert_eq!(datatype.as_rule(), Rule::datatype);

        match datatype.as_str() {
            "number" => Datatype::Number,
            "text" => Datatype::Text,
            "boolean" => Datatype::Boolean,
            _ => unreachable!(),
        }
    }
}
