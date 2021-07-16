use std::collections::HashMap;

use super::Table;
use crate::execute::{QueryResult, SelectQueryResult, SelectQueryResultColumn};
use crate::new_alias_map;
use crate::parse::ast;
use crate::Query;
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

    pub fn execute_old(&mut self, query: Query) -> QueryResult {
        match query {
            Query::SelectQuery(query) => {
                let table = self.find_table(query.table.root_table.name.0);
                let mut result = SelectQueryResult {
                    columns: table
                        .columns
                        .iter()
                        .map(|c| {
                            SelectQueryResultColumn::from(
                                query.table.root_table.name.0.to_owned(),
                                c,
                            )
                        })
                        .collect(),
                    rows: table.rows.clone(),
                    table_alias_map: new_alias_map(&query.table.root_table),
                };

                for join in query.table.joins {
                    let table = self.find_table(join.table.name.0);
                    let mut table = SelectQueryResult {
                        columns: table
                            .columns
                            .iter()
                            .map(|c| SelectQueryResultColumn::from(join.table.name.0.to_owned(), c))
                            .collect(),
                        rows: table.rows.clone(),
                        table_alias_map: new_alias_map(&join.table),
                    };
                    result.join(&mut table, &join);
                }

                if let Some(filter) = &query.filter {
                    result.filter(filter);
                }

                if let Some(sort) = &query.sort {
                    result.sort(sort);
                }

                if let Some(rows) = &query.limit {
                    // XXX evaluate, need to re-think this
                    result.limit(evaluate(rows).as_number());
                }

                let result = result.select(query.select_list);

                QueryResult::SelectQueryResult(result)
            }
            _ => unimplemented!(),
        }
    }
}

// TODO remove me
fn evaluate(x: &ast::Expression<'_>) -> Value {
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
