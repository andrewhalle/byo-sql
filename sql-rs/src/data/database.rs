use std::fs;
use std::io::{self, BufRead, Write};
use std::path::PathBuf;
use std::process;

use ansi_term::Colour;

use super::Column;
use super::Row;
use super::Table;
use crate::new_alias_map;
use crate::parse::ast;
use crate::parse::parse_queries;
use crate::CreateTableQueryResult;
use crate::InsertQueryResult;
use crate::Query;
use crate::QueryResult;
use crate::SelectQueryResult;
use crate::SelectQueryResultColumn;
use crate::Value;

pub struct Database {
    tables: Vec<Table>,
}

impl Database {
    pub fn new() -> Self {
        Database { tables: Vec::new() }
    }

    fn find_table(&self, table: &str) -> &Table {
        self.tables.iter().find(|t| t.name == table).unwrap()
    }

    fn find_table_mut(&mut self, table: &str) -> &mut Table {
        self.tables.iter_mut().find(|t| t.name == table).unwrap()
    }

    fn execute(&mut self, query: Query) -> QueryResult {
        match query {
            Query::SelectQuery(query) => {
                let table = self.find_table(query.table.root_table.name.0);
                let mut result = SelectQueryResult {
                    columns: table
                        .columns
                        .iter()
                        .map(|c| SelectQueryResultColumn::from(table.name.clone(), c))
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
                            .map(|c| SelectQueryResultColumn::from(table.name.clone(), c))
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
            Query::InsertQuery(query) => {
                if query.columns.len() != query.values.len() {
                    panic!();
                }

                let table = self.find_table_mut(query.table.0);
                let mut indices = table
                    .validate_insert_query_columns(
                        &(query.columns.iter().map(|i| i.0).collect::<Vec<&str>>()),
                    )
                    .unwrap();
                let mut row = table.new_values_vec();
                let mut values = query.values;

                while values.len() != 0 {
                    let i = indices.pop().unwrap();
                    let value = values.pop().unwrap();

                    let value = (&value).into();
                    if !table.compatible_type(i, &value) {
                        panic!();
                    }

                    row[i] = value;
                }

                table.rows.push(Row(row));

                QueryResult::InsertQueryResult(InsertQueryResult { num_inserted: 1 })
            }
            Query::CreateTableQuery(query) => {
                // TODO check that table doesn't exist
                let table = Table {
                    name: query.table_name.0.to_owned(),
                    columns: query
                        .columns
                        .iter()
                        .map(|c| Column {
                            name: c.name.0.to_owned(),
                            datatype: c.datatype,
                        })
                        .collect(),
                    rows: Vec::new(),
                };
                self.tables.push(table);
                QueryResult::CreateTableQueryResult(CreateTableQueryResult)
            }
        }
    }

    pub fn console(&mut self) {
        let stdin = io::stdin();
        let mut stdout = io::stdout();
        let mut reader = stdin.lock();
        let mut line = String::new();

        loop {
            line.clear();
            print!("> ");
            stdout.flush().unwrap();
            reader.read_line(&mut line).unwrap();

            if line == "" {
                println!();
                break;
            }

            let queries = parse_queries(&line);

            match queries {
                Ok(queries) => {
                    for query in queries.0 {
                        let result = self.execute(query);
                        println!("{}", result);
                    }
                }
                Err(parse_error) => {
                    println!("{}", parse_error);
                }
            }
        }
    }

    pub fn seed(&mut self, seed_file: PathBuf) {
        let seed = fs::read_to_string(seed_file).unwrap();
        let queries = parse_queries(&seed);

        match queries {
            Ok(queries) => {
                for query in queries.0 {
                    self.execute(query);
                }
            }
            Err(parse_error) => {
                println!("{}", parse_error);
                process::exit(1);
            }
        }

        let style = Colour::Fixed(251).italic();
        println!("{}\n\n{}", style.paint("Seeded with:"), style.paint(&seed));
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
