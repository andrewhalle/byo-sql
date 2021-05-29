#[macro_use]
extern crate pest_derive;
use pest::iterators::Pair;
use pest::Parser;

use std::fmt::{Display, Formatter};
use std::io::{self, BufRead, Write};

#[derive(Parser)]
#[grammar = "query.pest"]
pub struct QueryParser;

struct Database {
    tables: Vec<Table>,
}

struct Table {
    name: String,
    columns: Vec<Column>,
    rows: Vec<Row>,
}

#[derive(Clone)]
struct Column {
    name: String,
    datatype: Datatype,
}

// TODO this and Value are very similar
#[derive(Clone)]
enum Datatype {
    Number,
    Text,
}

// TODO this and Datatype are very similar
#[derive(Debug, Clone)]
enum Value {
    Number(u32),
    Text(String),
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Value::Number(n) => write!(f, "{}", n),
            Value::Text(s) => write!(f, "{}", s),
        }
    }
}

#[derive(Clone)]
struct Row(Vec<Value>);

#[derive(Debug)]
enum Query<'a> {
    SelectQuery(SelectQuery<'a>),
    InsertQuery(InsertQuery<'a>),
}

#[derive(Debug)]
struct SelectQuery<'a> {
    select_list: Vec<&'a str>,
    table: &'a str,
}

impl<'a> SelectQuery<'a> {
    fn from(select_query: Pair<'a, Rule>) -> Self {
        let mut inner = select_query.into_inner();

        let select_list = inner.next().unwrap();
        let select_list = {
            let inner: Vec<_> = select_list.into_inner().collect();
            if inner.len() == 0 {
                vec!["*"]
            } else {
                inner.iter().map(|identifier| identifier.as_str()).collect()
            }
        };

        let table = inner.next().unwrap().as_str();

        SelectQuery { select_list, table }
    }
}

#[derive(Debug)]
struct InsertQuery<'a> {
    table: &'a str,
    column_list: Vec<&'a str>,
    values: Vec<Value>,
}

impl<'a> InsertQuery<'a> {
    fn from(insert_query: Pair<'a, Rule>) -> Self {
        dbg!(&insert_query);

        // TODO

        InsertQuery {
            table: "",
            column_list: Vec::new(),
            values: Vec::new(),
        }
    }
}

impl<'a> Query<'a> {
    fn from(source: &'a str) -> Self {
        let parse = QueryParser::parse(Rule::query, source);

        let query = parse.unwrap().next().unwrap();
        let query = query.into_inner().next().unwrap();
        match query.as_rule() {
            Rule::select_query => Query::SelectQuery(SelectQuery::from(query)),
            Rule::insert_query => Query::InsertQuery(InsertQuery::from(query)),
            _ => unreachable!(),
        }
    }
}

struct QueryResult {
    columns: Vec<Column>,
    rows: Vec<Row>,
}

impl QueryResult {
    fn select(&mut self, columns: Vec<&str>) {
        if columns.len() == 1 && columns[0] == "*" {
            return;
        } else {
            let indices: Vec<usize> = columns
                .iter()
                .map(|c| {
                    self.columns
                        .iter()
                        .map(|c| &c.name)
                        .enumerate()
                        .find(|(_, x)| x == c)
                        .unwrap()
                        .0
                })
                .collect();

            let mut new_columns = Vec::new();
            for i in indices.iter() {
                new_columns.push(self.columns[*i].clone());
            }
            self.columns = new_columns;

            for row in self.rows.iter_mut() {
                let mut new_row = Vec::new();
                for i in indices.iter() {
                    new_row.push(row.0[*i].clone());
                }
                *row = Row(new_row);
            }

            return;
        }
    }
}

impl Display for QueryResult {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        let column_names: Vec<&str> = self.columns.iter().map(|c| c.name.as_str()).collect();
        writeln!(f, "{}", &column_names.join(",")).unwrap();

        let num_rows = self.rows.len();
        for (i, row) in self.rows.iter().enumerate() {
            let values: Vec<String> = row.0.iter().map(|v| v.to_string()).collect();
            write!(f, "{}", &values.join(",")).unwrap();
            if i != num_rows - 1 {
                writeln!(f).unwrap();
            }
        }

        Ok(())
    }
}

impl Database {
    fn new() -> Self {
        Database { tables: Vec::new() }
    }

    // TODO make this generic over a Rust struct implementing something like a IntoTable trait
    // currently adds a fixed user table with a fixed row
    fn add_table(&mut self) {
        let user1 = vec![
            Value::Text("1".to_string()),
            Value::Text("user1@email.com".to_string()),
            Value::Number(25),
        ];
        let user2 = vec![
            Value::Text("2".to_string()),
            Value::Text("user2@email.com".to_string()),
            Value::Number(27),
        ];

        self.tables.push(Table {
            name: "users".to_string(),
            rows: vec![Row(user1), Row(user2)],
            columns: vec![
                Column {
                    name: "id".to_string(),
                    datatype: Datatype::Text,
                },
                Column {
                    name: "email".to_string(),
                    datatype: Datatype::Text,
                },
                Column {
                    name: "age".to_string(),
                    datatype: Datatype::Number,
                },
            ],
        });
    }

    fn execute(&mut self, query: Query) -> QueryResult {
        let table = &self.tables[0];
        let mut result = QueryResult {
            columns: table.columns.clone(),
            rows: table.rows.clone(),
        };

        match query {
            Query::SelectQuery(query) => {
                result.select(query.select_list);
                result
            }
            _ => todo!(),
        }
    }

    fn console(&mut self) {
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
            let result = self.execute(query);
            println!("{}", result);
        }
    }
}

#[derive(Debug)]
struct User {
    id: String,
    email: String,
    age: u32,
}

fn main() {
    let mut db = Database::new();
    db.add_table();
    db.console();
}
