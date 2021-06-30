use super::Listable;

/// A literal value.
///
/// Possible literals are string literals, number literals, and boolean literals.
// TODO consider removing this type, and using the Value type from the data module, leaving the
// implementation of From<Pair<..>> here
#[derive(Debug)]
pub enum Literal<'input> {
    String(&'input str),
    Number(&'input str),
    Boolean(&'input str),
}

use crate::parse::Rule;
use pest::iterators::Pair;

impl<'input> From<Pair<'input, Rule>> for Literal<'input> {
    fn from(literal: Pair<'input, Rule>) -> Self {
        assert_eq!(literal.as_rule(), Rule::literal);

        let inner_literal = literal.into_inner().next().unwrap();

        match inner_literal.as_rule() {
            Rule::string_literal => {
                let string_contents = inner_literal.into_inner().next().unwrap();

                Literal::String(string_contents.as_str())
            }
            Rule::number_literal => Literal::Number(inner_literal.as_str()),
            Rule::boolean_literal => Literal::Boolean(inner_literal.as_str()),
            _ => unreachable!(),
        }
    }
}

impl<'input> Listable for Literal<'input> {
    fn get_rule() -> Rule {
        Rule::literal_list
    }
}

use crate::Value;
impl<'input> From<Literal<'input>> for Value {
    fn from(literal: Literal<'input>) -> Self {
        match literal {
            Literal::String(s) => Value::Text(s.to_owned()),
            Literal::Number(n) => Value::Number(n.parse().unwrap()),
            Literal::Boolean(b) => Value::Boolean(b.parse().unwrap()),
        }
    }
}
