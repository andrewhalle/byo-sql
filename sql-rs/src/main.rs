use std::path::PathBuf;

use structopt::StructOpt;

use sql_rs::data::Database;

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(short, parse(from_os_str))]
    seed_file: Option<PathBuf>,
}

fn main() {
    let opt = Opt::from_args();

    let mut db = Database::new();
    if let Some(seed_file) = opt.seed_file {
        db.seed(seed_file);
    }

    db.console();
}
