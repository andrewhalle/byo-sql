use std::collections::HashSet;

use super::{Column, Row, Value};

/// TODO short description.
///
/// TODO long description.
pub struct Table {
    pub columns: Vec<Column>,
    pub rows: Vec<Row>,
}

impl Table {
    pub fn validate_insert_query_columns(
        &self,
        insert_query_columns: &[&str],
    ) -> Option<Vec<usize>> {
        // first check that all columns are provided
        {
            let self_columns: HashSet<&str> =
                self.columns.iter().map(|c| c.name.as_str()).collect();
            let insert_query_columns: HashSet<&str> =
                insert_query_columns.iter().map(|c| *c).collect();

            if self_columns != insert_query_columns {
                return None;
            }
        }

        // then map the insert_query_column to the index in the table specificiation
        let self_columns: Vec<(usize, &str)> = self
            .columns
            .iter()
            .map(|c| c.name.as_str())
            .enumerate()
            .collect();
        let indices = insert_query_columns
            .iter()
            .map(|c1| self_columns.iter().find(|(_, c2)| c1 == c2).unwrap().0)
            .collect();
        Some(indices)
    }

    pub fn new_values_vec(&self) -> Vec<Value> {
        vec![Value::Null; self.columns.len()]
    }

    pub fn compatible_type(&self, column_index: usize, value: &Value) -> bool {
        let column = &self.columns[column_index];
        value.assignable_to(column.datatype)
    }
}
