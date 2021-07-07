use crate::parse::ast::ExpressionOp;

/// TODO short description.
///
/// TODO long description.
// TODO remove PartialEq and Eq
// TODO this and Datatype are very similar
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    Null,
    Number(u32),
    Text(String),
    Boolean(bool),
}

impl Value {
    pub fn op(self, op: ExpressionOp, rhs: Value) -> Value {
        match op {
            ExpressionOp::GreaterEqual => Value::Boolean(self.greater_equal(&rhs)),
            ExpressionOp::LessEqual => Value::Boolean(self.less_equal(&rhs)),
            ExpressionOp::Greater => Value::Boolean(self.greater(&rhs)),
            ExpressionOp::Less => Value::Boolean(self.less(&rhs)),
            ExpressionOp::And => Value::Boolean(self.is_true() && rhs.is_true()),
            ExpressionOp::Or => Value::Boolean(self.is_true() || rhs.is_true()),
            ExpressionOp::Equal => Value::Boolean(self == rhs),
            ExpressionOp::Plus => Value::Number(self.as_number() + rhs.as_number()),
            ExpressionOp::Minus => Value::Number(self.as_number() - rhs.as_number()),
        }
    }
}
