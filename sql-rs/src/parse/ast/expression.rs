use lazy_static::lazy_static;

use pest::prec_climber::{Operator, PrecClimber};

use super::{ColumnIdentifier, Literal};

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
        );

        match rule {
            greater_equal => ExpressionOp::GreaterEqual,
            less_equal => ExpressionOp::LessEqual,
            greater => ExpressionOp::Greater,
            less => ExpressionOp::Less,
            and => ExpressionOp::And,
            or => ExpressionOp::Or,
            equal => ExpressionOp::Equal,
            _ => unreachable!(),
        }
    }
}

/*
   fn evaluate(&self, expr: Expression<'_>, row: &Row) -> Value {
       PREC_CLIMBER.climb(
           expr.inner.into_inner(),
           |pair: Pair<Rule>| match pair.as_rule() {
               Rule::column_identifier => {
                   self.get_column_value_from_row(ColumnIdentifier::from(pair), row)
               }
               Rule::expression => self.evaluate(
                   Expression {
                       inner: pair.clone(),
                   },
                   row,
               ),
               Rule::literal => Value::from(pair),
               _ => unreachable!(),
           },
           |lhs: Value, op: Pair<Rule>, rhs: Value| match op.as_rule() {
               Rule::greater_equal => Value::Boolean(lhs.greater_equal(&rhs)),
               Rule::less_equal => Value::Boolean(lhs.less_equal(&rhs)),
               Rule::greater => Value::Boolean(lhs.greater(&rhs)),
               Rule::less => Value::Boolean(lhs.less(&rhs)),
               Rule::and => Value::Boolean(lhs.is_true() && rhs.is_true()),
               Rule::or => Value::Boolean(lhs.is_true() || rhs.is_true()),
               Rule::equal => Value::Boolean(lhs == rhs),
               _ => unreachable!(),
           },
       )
   }
*/
