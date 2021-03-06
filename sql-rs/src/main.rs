use std::fs;
use std::path::PathBuf;
use std::process;

use structopt::StructOpt;

use ansi_term::Colour;

use rustyline::error::ReadlineError;
use rustyline::Editor;

use sql_rs::data::Database;
use sql_rs::parse::parse_queries;

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(short, parse(from_os_str))]
    seed_file: Option<PathBuf>,
}

pub fn seed(database: &mut Database, seed_file: PathBuf) {
    let seed = fs::read_to_string(seed_file).unwrap();
    let queries = parse_queries(&seed);

    match queries {
        Ok(queries) => {
            for query in queries.0 {
                database.execute(query).unwrap();
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

pub fn console(database: &mut Database) {
    let mut rl = Editor::<()>::new();

    loop {
        let readline = rl.readline("> ");
        match readline {
            Ok(line) => {
                let queries = parse_queries(&line);

                match queries {
                    Ok(queries) => {
                        for query in queries.0 {
                            match database.execute(query) {
                                Ok(s) => println!("{}", s),
                                Err(e) => println!("{}", e),
                            }
                        }
                    }
                    Err(parse_error) => {
                        println!("{}", parse_error);
                    }
                }

                rl.add_history_entry(line);
            }
            Err(ReadlineError::Eof) => {
                println!();
                break;
            }
            _ => panic!(),
        }
    }
}

fn main() {
    let opt = Opt::from_args();

    let mut db = Database::new();
    if let Some(seed_file) = opt.seed_file {
        seed(&mut db, seed_file);
    }

    console(&mut db);
}
