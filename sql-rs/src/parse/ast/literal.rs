/// A literal value.
///
/// Possible literals are string literals, number literals, and boolean literals.
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
