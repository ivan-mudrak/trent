use crate::engine::csv::process_csv_file;
use std::{env, process::exit};

mod engine;
mod readers;
mod types;

fn main() {
    let args = env::args().collect::<Vec<String>>();

    if args.len() < 2 {
        println!("no input CSV file");
    }

    let file_path = &args[1];

    let start = std::time::SystemTime::now();
    match process_csv_file(file_path, std::io::stdout()) {
        Ok(_) => {
            let end = std::time::SystemTime::now();
            println!("elapsed: {:?}", end.duration_since(start));
        }
        Err(e) => {
            println!("error: {}", e);
            exit(1);
        }
    }
}
