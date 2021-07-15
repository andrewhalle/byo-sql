use std::collections::HashMap;
use std::fs;
use std::io::{self, BufRead, Write};
use std::path::PathBuf;
use std::process;

use ansi_term::Colour;

use super::Row;
use super::Table;
use crate::execute::{InsertQueryResult, QueryResult, SelectQueryResult, SelectQueryResultColumn};
use crate::new_alias_map;
use crate::parse::ast;
use crate::parse::parse_queries;
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

    fn find_table(&self, table: &str) -> &Table {
        self.tables.get(table).unwrap()
    }

    fn find_table_mut(&mut self, table: &str) -> &mut Table {
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
            _ => unimplemented!(),
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
                        match self.execute(query) {
                            Ok(s) => println!("{}", s),
                            Err(e) => println!("{}", e),
                        }
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
