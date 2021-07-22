use std::collections::HashMap;

use super::Table;

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
