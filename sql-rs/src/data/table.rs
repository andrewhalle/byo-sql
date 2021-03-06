use std::collections::HashSet;
use std::fmt::{Display, Formatter};
use std::mem;

use super::{Column, Row, Value};
use crate::execute::RowEvaluationContext;
use crate::parse::ast::{ColumnIdentifier, JoinKind};

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
                let name = {
                    let mut parts = column.name.split(".");

                    let first = parts.next().unwrap();
                    let second = parts.next();

                    match second {
                        Some(name) => name,
                        None => first,
                    }
                };

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

    pub fn filter_mut<F: Fn(RowEvaluationContext) -> bool>(
        &mut self,
        predicate: F,
    ) -> Vec<&mut Row> {
        let mut columns = Vec::new();
        mem::swap(&mut columns, &mut self.columns);

        let mut rows: Vec<&mut Row> = self.rows.iter_mut().collect();
        rows.retain(|row| predicate((&columns, row)));

        mem::swap(&mut columns, &mut self.columns);

        rows
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

    // TODO re-write. make cleaner.
    // Nested loop join.
    pub fn join<F: Fn(RowEvaluationContext) -> bool>(
        &mut self,
        mut rhs: Table,
        predicate: F,
        join_kind: JoinKind,
    ) {
        let lhs_column_count = self.columns.len();
        let rhs_column_count = rhs.columns.len();

        self.columns.append(&mut rhs.columns);

        let mut columns = Vec::new();
        mem::swap(&mut columns, &mut self.columns);

        let outer_iter = || match join_kind {
            JoinKind::Right => rhs.rows.iter(),
            _ => self.rows.iter(),
        };
        let inner_iter = || match join_kind {
            JoinKind::Right => self.rows.iter(),
            _ => rhs.rows.iter(),
        };

        let mut rows = Vec::new();
        for i in outer_iter() {
            let mut did_add_row = false;

            for j in inner_iter() {
                let mut row = i.0.clone();
                row.append(&mut j.0.clone());
                let row = Row(row);
                if predicate((&columns, &row)) {
                    rows.push(row);
                    did_add_row = true;
                }
            }

            if !did_add_row {
                let row = match join_kind {
                    JoinKind::Left => {
                        let mut row = i.0.clone();
                        let mut nulls = {
                            let mut nulls = Vec::with_capacity(rhs_column_count);
                            for _i in 0..rhs_column_count {
                                nulls.push(Value::Null);
                            }
                            nulls
                        };
                        row.append(&mut nulls);
                        Some(Row(row))
                    }
                    JoinKind::Right => {
                        let mut row = i.0.clone();
                        let mut nulls = {
                            let mut nulls = Vec::with_capacity(lhs_column_count);
                            for _i in 0..lhs_column_count {
                                nulls.push(Value::Null);
                            }
                            nulls
                        };
                        nulls.append(&mut row);
                        Some(Row(nulls))
                    }
                    _ => None,
                };

                if row.is_some() {
                    rows.push(row.unwrap());
                }
            }
        }

        mem::swap(&mut self.rows, &mut rows);
        mem::swap(&mut columns, &mut self.columns);
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

    pub fn get_update_indices(&self, cols: &[&str]) -> Vec<usize> {
        let self_columns: Vec<(usize, &str)> = self
            .columns
            .iter()
            .map(|c| c.name.as_str())
            .enumerate()
            .collect();
        let indices = cols
            .iter()
            .map(|c1| self_columns.iter().find(|(_, c2)| c1 == c2).unwrap().0)
            .collect();

        indices
    }
}

impl Display for Table {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        // write columns
        let columns: Vec<_> = self.columns.iter().map(|v| v.to_string()).collect();
        let columns: Vec<_> = columns.iter().map(|s| s.as_str()).collect();
        writeln!(f, "{}", columns.join(","))?;

        // write rows
        if self.rows.len() > 0 {
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
        }

        Ok(())
    }
}
