use std::fmt::{Display, Formatter};

use crate::data::Datatype;
use crate::parse::ast::ExpressionOp;

/// TODO short description.
///
/// TODO long description.
// TODO remove PartialEq and Eq
// TODO this and Datatype are very similar
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Value {
    Null,
    Number(u32),
    Text(String),
    Boolean(bool),
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        match self {
            Value::Number(n) => write!(f, "{}", n),
            Value::Text(s) => write!(f, "{}", s),
            Value::Boolean(b) => write!(f, "{}", b),
            Value::Null => write!(f, "null"),
        }
    }
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

    pub fn assignable_to(&self, datatype: Datatype) -> bool {
        match self {
            Value::Null => true,
            Value::Number(_) => datatype == Datatype::Number,
            Value::Text(_) => datatype == Datatype::Text,
            Value::Boolean(_) => datatype == Datatype::Boolean,
        }
    }

    pub fn is_true(&self) -> bool {
        match self {
            Value::Boolean(b) => *b,
            _ => panic!("cannot use a non-boolean Value in a boolean context"),
        }
    }

    pub fn as_number(&self) -> u32 {
        match self {
            Value::Number(n) => *n,
            _ => panic!(),
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Value::Text(s) => s.as_str(),
            _ => panic!(),
        }
    }
}

macro_rules! value_op {
    ($name:ident, $op:tt) => {
        impl Value {
            fn $name(&self, rhs: &Value) -> bool {
                match self {
                    Value::Text(_) => self.as_str() $op rhs.as_str(),
                    Value::Number(_) => self.as_number() $op rhs.as_number(),
                    _ => panic!("operands of $op must be text or number"),
                }
            }
        }
    };
}

value_op!(greater_equal, >=);
value_op!(greater, >);
value_op!(less_equal, <=);
value_op!(less, <);
