use std::cmp::Reverse;
use std::fmt::{Display, Formatter};

use super::{evaluate, evaluate_column};
use crate::data::{Database, Row, Table};
use crate::parse::ast::{self, Expression, OrderByDirection, SelectQuery};

pub type Success = Table;

#[derive(Debug)]
pub struct Error;

impl Display for Error {
    fn fmt(&self, _f: &mut Formatter) -> Result<(), std::fmt::Error> {
        todo!()
    }
}

type QueryResult = Result<Success, Error>;

impl Database {
    fn queried_tables(&self, query: &SelectQuery<'_>) -> Table {
        // start with the root table
        let mut result = self.find_table(query.table.root_table.name.0).clone();
        result.prefix_column_names(&format!("{}.", query.table.root_table.as_str()));

        // add all joined tables
        for join in &query.table.joins {
            let mut table = self.find_table(join.table.name.0).clone();
            table.prefix_column_names(&format!("{}.", join.table.as_str()));

            result.join(
                table,
                |evaluation_context| {
                    evaluate(&join.condition, Some(evaluation_context), None).is_true()
                },
                join.kind,
            );
        }

        result
    }

    fn apply_query_transformations(&self, query: &SelectQuery<'_>, result: &mut Table) {
        if let Some(filter) = &query.filter {
            result.filter(|evaluation_context| {
                evaluate(filter, Some(evaluation_context), None).is_true()
            });
        }

        if let Some(sort) = &query.sort {
            match sort.direction {
                OrderByDirection::Asc => result.sort(|evaluation_context| {
                    evaluate(&sort.expr, Some(evaluation_context), None)
                }),
                OrderByDirection::Desc => result.sort(|evaluation_context| {
                    Reverse(evaluate(&sort.expr, Some(evaluation_context), None))
                }),
            }
        }

        if let Some(rows) = &query.limit {
            result.limit(evaluate(rows, None, None).as_number() as usize);
        }
    }

    pub fn execute_select(&self, query: SelectQuery<'_>) -> QueryResult {
        let mut result = self.queried_tables(&query);
        self.apply_query_transformations(&query, &mut result);

        Ok(apply_selection(&query, &result))
    }
}

fn apply_selection(query: &SelectQuery<'_>, result: &Table) -> Table {
    // generate the columns of the new table
    let new_columns: Vec<_> = query
        .select_list
        .iter()
        .flat_map(|expr| evaluate_column(expr, &result.columns))
        .collect();

    // generate the rows of the new table
    let mut new_rows = Vec::new();
    for row in &result.rows {
        // TODO at this point, we know how many values there are going to be. can
        // pre-allocate space
        let mut new_row = Vec::new();

        for expr in &query.select_list {
            match expr {
                Expression::ColumnIdentifier(
                    i
                    @
                    ast::ColumnIdentifier {
                        name: ast::Column::Star,
                        ..
                    },
                ) => {
                    let mut values = (&result.columns)
                        .iter()
                        .enumerate()
                        .filter(|(_, c)| match &i.alias {
                            None => true,
                            Some(alias) => {
                                if c.name.contains(".") {
                                    alias.0 == c.name.rsplit_once(".").unwrap().0
                                } else {
                                    alias.0 == c.name
                                }
                            }
                        })
                        .map(|(idx, _)| row.0[idx].clone())
                        .collect();
                    new_row.append(&mut values);
                }
                _ => new_row.push(evaluate(expr, Some((&result.columns, row)), None)),
            }
        }

        new_rows.push(Row(new_row));
    }

    Table {
        columns: new_columns,
        rows: new_rows,
    }
}
