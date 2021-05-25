use std::fmt::{Display, Formatter};
use std::io::{self, BufRead, Write};

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
#[derive(Clone)]
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

struct Query;

impl Query {
    // TODO actually parse SQL
    fn from(source: &str) -> Self {
        Query
    }
}

struct QueryResult {
    columns: Vec<Column>,
    rows: Vec<Row>,
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

        QueryResult {
            columns: table.columns.clone(),
            rows: table.rows.clone(),
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
