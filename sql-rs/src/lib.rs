//! A crate for parsing and executing SQL in-memory against a simple database representation.

use std::cmp::Ordering;
use std::fmt::{Display, Formatter};

#[macro_use]
extern crate pest_derive;

/// Data representation.
pub mod data;
use data::{Datatype, Value};

/// Executing a query against a database.
pub mod execute;

/// Parsing SQL.
pub mod parse;

/*
fn cmp_column_with_column_identifier(
    c1: &SelectQueryResultColumn,
    c2: &ColumnIdentifier<'_>,
    table_alias_map: &HashMap<String, String>,
) -> bool {
    let is_star = matches!(c2.name, ast::Column::Star);
    let c2_column = match &c2.name {
        ast::Column::Star => None,
        ast::Column::Ident(i) => Some(i.0),
    };
    let names_match = is_star || c1.column == c2_column.unwrap();

    match c2.alias.as_ref() {
        None => names_match,
        Some(alias) => &c1.table == table_alias_map.get(alias.0).unwrap() && names_match,
    }
}
*/

#[derive(PartialEq, Eq)]
struct SortableValue(Value);

// TODO can this implementation be derived?
impl Ord for SortableValue {
    fn cmp(&self, other: &Self) -> Ordering {
        match &self.0 {
            Value::Null => match &other.0 {
                Value::Null => Ordering::Equal,
                _ => Ordering::Less,
            },
            Value::Number(n1) => match &other.0 {
                Value::Null => Ordering::Greater,
                Value::Number(n2) => n1.cmp(&n2),
                _ => Ordering::Less,
            },
            Value::Text(s1) => match &other.0 {
                Value::Null | Value::Number(_) => Ordering::Greater,
                Value::Text(s2) => s1.cmp(&s2),
                _ => Ordering::Less,
            },
            Value::Boolean(b1) => match &other.0 {
                Value::Null | Value::Number(_) | Value::Text(_) => Ordering::Greater,
                Value::Boolean(b2) => b1.cmp(&b2),
            },
        }
    }
}

impl PartialOrd for SortableValue {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Value {
    fn assignable_to(&self, datatype: Datatype) -> bool {
        match self {
            Value::Null => true,
            Value::Number(_) => datatype == Datatype::Number,
            Value::Text(_) => datatype == Datatype::Text,
            Value::Boolean(_) => datatype == Datatype::Boolean,
        }
    }

    fn is_true(&self) -> bool {
        match self {
            Value::Boolean(b) => *b,
            _ => panic!("cannot use a non-boolean Value in a boolean context"),
        }
    }

    fn as_number(&self) -> u32 {
        match self {
            Value::Number(n) => *n,
            _ => panic!(),
        }
    }

    fn as_str(&self) -> &str {
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

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Value::Number(n) => write!(f, "{}", n),
            Value::Text(s) => write!(f, "{}", s),
            Value::Boolean(b) => write!(f, "{}", b),
            Value::Null => write!(f, "null"),
        }
    }
}

/*
impl<'input> Expression<'input> {
    // TODO make better / re-locate
    fn is_column_star(&self) -> bool {
        matches!(
            self,
            ast::Expression::ColumnIdentifier(ColumnIdentifier {
                name: ast::Column::Star,
                ..
            })
        )
    }

    // TODO make better / re-locate
    fn is_count(&self) -> bool {
        matches!(self, ast::Expression::CountStar)
    }
}
*/
