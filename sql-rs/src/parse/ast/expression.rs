use lazy_static::lazy_static;

use pest::prec_climber::{Operator, PrecClimber};

use super::{ColumnIdentifier, Listable, Literal};

/// An expression that can be evaluated.
///
/// An AST version of expressions for SQL. Parsed from tokens using a PrecClimber. In this form,
/// the rest of the code can evaluate expressions without requiring knowledge of pest.
#[derive(Debug)]
pub enum Expression<'input> {
    /// Hack to get the most common aggregate function working.
    CountStar,
    Literal(Literal<'input>),
    ColumnIdentifier(ColumnIdentifier<'input>),
    BinaryOp(BinaryOp<'input>),
}

#[derive(Debug)]
pub struct BinaryOp<'input> {
    op: ExpressionOp,
    left: Box<Expression<'input>>,
    right: Box<Expression<'input>>,
}

#[derive(Debug)]
enum ExpressionOp {
    GreaterEqual,
    LessEqual,
    Greater,
    Less,
    And,
    Or,
    Equal,
    Plus,
    Minus,
}

lazy_static! {
    static ref PREC_CLIMBER: PrecClimber<Rule> = {
        use pest::prec_climber::Assoc::*;
        use Rule::*;

        PrecClimber::new(vec![
            Operator::new(and, Left) | Operator::new(or, Left),
            Operator::new(greater_equal, Left)
                | Operator::new(greater, Left)
                | Operator::new(less_equal, Left)
                | Operator::new(less, Left)
                | Operator::new(equal, Left),
            Operator::new(plus, Left) | Operator::new(minus, Left),
        ])
    };
}

use crate::parse::Rule;
use pest::iterators::Pair;

impl<'input> From<Pair<'input, Rule>> for Expression<'input> {
    fn from(expression: Pair<'input, Rule>) -> Self {
        assert_eq!(expression.as_rule(), Rule::expression);

        PREC_CLIMBER.climb(
            expression.into_inner(),
            |pair: Pair<Rule>| match pair.as_rule() {
                Rule::column_identifier => Expression::ColumnIdentifier(pair.into()),
                Rule::expression => pair.into(),
                Rule::literal => Expression::Literal(pair.into()),
                Rule::count_star => Expression::CountStar,
                _ => unreachable!(),
            },
            |left: Expression<'_>, op: Pair<Rule>, right: Expression<'_>| {
                Expression::BinaryOp(BinaryOp {
                    left: Box::new(left),
                    op: op.into(),
                    right: Box::new(right),
                })
            },
        )
    }
}

impl From<Pair<'_, Rule>> for ExpressionOp {
    fn from(operation: Pair<'_, Rule>) -> Self {
        use Rule::*;

        let rule = operation.as_rule();
        assert!(
            rule == greater_equal
                || rule == less_equal
                || rule == greater
                || rule == less
                || rule == and
                || rule == or
                || rule == equal
                || rule == plus
                || rule == minus
        );

        match rule {
            greater_equal => ExpressionOp::GreaterEqual,
            less_equal => ExpressionOp::LessEqual,
            greater => ExpressionOp::Greater,
            less => ExpressionOp::Less,
            and => ExpressionOp::And,
            or => ExpressionOp::Or,
            equal => ExpressionOp::Equal,
            plus => ExpressionOp::Plus,
            minus => ExpressionOp::Minus,
            _ => unreachable!(),
        }
    }
}

impl<'input> Listable for Expression<'input> {
    fn get_rule() -> Rule {
        Rule::expression_list
    }
}
