use crate::data::Datatype;
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
