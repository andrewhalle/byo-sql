use std::io::{self, BufRead, Write};

use sql_rs::parse::parse_queries;

fn main() {
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
        dbg!(&queries);

        // use result
        match queries {
            _ => {}
        }
    }
}
