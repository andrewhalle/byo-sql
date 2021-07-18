use std::collections::HashMap;

use super::Table;
use crate::parse::ast;
use crate::Value;

pub struct Database {
    pub tables: HashMap<String, Table>,
}

impl Database {
    pub fn new() -> Self {
        Database {
            tables: HashMap::new(),
        }
    }

    pub fn find_table(&self, table: &str) -> &Table {
        self.tables.get(table).unwrap()
    }

    pub fn find_table_mut(&mut self, table: &str) -> &mut Table {
        self.tables.get_mut(table).unwrap()
    }
}

// TODO remove me
pub fn evaluate(x: &ast::Expression<'_>) -> Value {
    match x {
        ast::Expression::Literal(l) => l.into(),
        ast::Expression::BinaryOp(b) => {
            let v1 = evaluate(&b.left);
            let v2 = evaluate(&b.right);

            v1.op(b.op, v2)
        }
        _ => unreachable!(),
    }
}
