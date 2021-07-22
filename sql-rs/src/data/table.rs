use std::collections::HashSet;
use std::fmt::{Display, Formatter};
use std::mem;

use super::{Column, Row, Value};
use crate::execute::RowEvaluationContext;
use crate::parse::ast::ColumnIdentifier;

/// TODO short description.
///
/// TODO long description.
#[derive(Debug, Clone)]
pub struct Table {
    pub columns: Vec<Column>,
    pub rows: Vec<Row>,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("union of tables requires columns of LHS and RHS to match")]
    ColumnMismatch,
}

// fundamental table operations

impl Table {
    pub fn get_column_idx(columns: &Vec<Column>, column_identifier: &ColumnIdentifier) -> usize {
        // TODO make sure that the column identifier uniquely specifies a column within the table
        let (column_idx, _) = columns
            .iter()
            .enumerate()
            .find(|(_, column)| {
                let mut parts = column.name.split(".");

                let _alias = parts.next().unwrap();
                let name = parts.next().unwrap();

                match &column_identifier.alias {
                    None => name == column_identifier.as_string(),
                    Some(_) => column.name == column_identifier.as_string(),
                }
            })
            .unwrap();

        column_idx
    }

    pub fn filter<F: Fn(RowEvaluationContext) -> bool>(&mut self, predicate: F) {
        let mut columns = Vec::new();
        mem::swap(&mut columns, &mut self.columns);

        self.rows.retain(|row| predicate((&columns, row)));

        mem::swap(&mut columns, &mut self.columns);
    }

    pub fn limit(&mut self, limit: usize) {
        self.rows.truncate(limit);
    }

    pub fn sort<K: Ord, F: Fn(RowEvaluationContext) -> K>(&mut self, key_fn: F) {
        let mut columns = Vec::new();
        mem::swap(&mut columns, &mut self.columns);

        self.rows
            .sort_unstable_by_key(|row| key_fn((&columns, row)));

        mem::swap(&mut columns, &mut self.columns);
    }

    // resulting table contains all rows of both tables
    pub fn union(&mut self, _other: &mut Table) -> Result<(), Error> {
        todo!()
    }
}

// utilities

impl Table {
    pub fn prefix_column_names(&mut self, prefix: &str) {
        for column in &mut self.columns {
            column.name.insert_str(0, prefix);
        }
    }
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

impl Display for Table {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        // write columns
        let columns: Vec<_> = self.columns.iter().map(|v| v.to_string()).collect();
        let columns: Vec<_> = columns.iter().map(|s| s.as_str()).collect();
        writeln!(f, "{}", columns.join(","))?;

        // write rows
        let num_rows = self.rows.len();
        for row in &self.rows[..num_rows - 1] {
            let values: Vec<_> = row.0.iter().map(|v| v.to_string()).collect();
            let values: Vec<_> = values.iter().map(|s| s.as_str()).collect();
            writeln!(f, "{}", values.join(","))?;
        }

        let values: Vec<_> = self.rows[num_rows - 1]
            .0
            .iter()
            .map(|v| v.to_string())
            .collect();
        let values: Vec<_> = values.iter().map(|s| s.as_str()).collect();
        write!(f, "{}", values.join(","))?;

        Ok(())
    }
}
