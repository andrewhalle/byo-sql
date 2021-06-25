use std::fs;
use std::io::{self, BufRead, Write};
use std::path::PathBuf;
use std::process;

use ansi_term::Colour;

use crate::new_alias_map;
use crate::CreateTableQueryResult;
use crate::InsertQueryResult;
use crate::Query;
use crate::QueryResult;
use crate::Row;
use crate::SelectQueryResult;
use crate::SelectQueryResultColumn;
use crate::Table;

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
                let table = self.find_table(query.table.root_table.name);
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
                    let table = self.find_table(join.table.name);
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
                    result.limit(*rows);
                }

                let result = result.select(query.select_list);

                QueryResult::SelectQueryResult(result)
            }
            Query::InsertQuery(query) => {
                if query.column_list.len() != query.values.len() {
                    panic!();
                }

                let table = self.find_table_mut(query.table);
                let mut indices = table
                    .validate_insert_query_columns(&query.column_list)
                    .unwrap();
                let mut row = table.new_values_vec();
                let mut values = query.values;

                while values.len() != 0 {
                    let i = indices.pop().unwrap();
                    let value = values.pop().unwrap();

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
                    name: query.table_name.to_owned(),
                    columns: query.columns,
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

            let query = Query::from(&line);

            match query {
                Ok(query) => {
                    let result = self.execute(query);
                    println!("{}", result);
                }
                Err(parse_error) => {
                    println!("{}", parse_error);
                }
            }
        }
    }

    pub fn seed(&mut self, seed_file: PathBuf) {
        let seed = fs::read_to_string(seed_file).unwrap();
        let queries = Query::get_many(&seed);

        match queries {
            Ok(queries) => {
                for query in queries {
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
