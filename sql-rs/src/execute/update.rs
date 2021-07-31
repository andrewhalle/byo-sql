use std::fmt::{Display, Formatter};

use crate::data::Database;
use crate::execute::evaluate;
use crate::parse::ast::UpdateQuery;

#[derive(Debug)]
pub struct Success {
    num_updated: u32,
}

impl Display for Success {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "UPDATE {}", self.num_updated)
    }
}

#[derive(Debug)]
pub enum Error {
    IncompatibleDatatype,
}

impl Display for Error {
    fn fmt(&self, _f: &mut Formatter) -> Result<(), std::fmt::Error> {
        todo!()
    }
}

type QueryResult = Result<Success, Error>;

impl Database {
    pub fn execute_update(&mut self, query: UpdateQuery<'_>) -> QueryResult {
        let table = self.find_table_mut(query.table.0);
        let assignment_names: Vec<_> = query.assignments.iter().map(|a| a.column.0).collect();
        let indices = table.get_update_indices(assignment_names.as_slice());
        let rows_to_update = table.filter_mut(|evaluation_context| {
            evaluate(&query.filter, Some(evaluation_context), None).is_true()
        });
        let num_updated = rows_to_update.len() as u32;

        for row in rows_to_update {
            let values = query.assignments.iter().map(|assignment| &assignment.value);
            for (&idx, value) in indices.iter().zip(values) {
                // TODO fix partial updates possible here after introducing error conditions
                // TODO fix possible to assign to incompatible data type
                row.0[idx] = value.into();
            }
        }

        Ok(Success { num_updated })
    }
}
