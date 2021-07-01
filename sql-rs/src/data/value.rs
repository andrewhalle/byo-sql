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
            ExpressionOp::Equal => Value::Boolean(self == rhs),
            _ => todo!(),
        }
    }
}
