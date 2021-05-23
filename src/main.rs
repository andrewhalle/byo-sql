use std::io::{self, BufRead, Write};

#[derive(Debug)]
struct User {
    id: String,
    email: String,
    first_name: String,
    last_name: String,
    age: u32,
}

struct Database(Vec<User>);

impl Database {
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

            print!("{}", line);
        }
    }
}

fn main() {
    let users = vec![
        User {
            id: "1".to_string(),
            email: "user1@test.com".to_string(),
            first_name: "User1".to_string(),
            last_name: "Person".to_string(),
            age: 14,
        },
        User {
            id: "2".to_string(),
            email: "user2@test.com".to_string(),
            first_name: "User2".to_string(),
            last_name: "Person".to_string(),
            age: 24,
        },
        User {
            id: "3".to_string(),
            email: "user3@test.com".to_string(),
            first_name: "User3".to_string(),
            last_name: "Person".to_string(),
            age: 34,
        },
        User {
            id: "4".to_string(),
            email: "user4@test.com".to_string(),
            first_name: "User4".to_string(),
            last_name: "Person".to_string(),
            age: 44,
        },
    ];

    let mut db = Database(users);
    db.console();
}
