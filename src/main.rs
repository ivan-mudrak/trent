use crate::engine::csv::process_csv_file;
use std::{env, process::exit};

mod engine;
mod readers;
mod types;

#[tokio::main]
async fn main() {
    let args = env::args().collect::<Vec<String>>();

    if args.len() < 2 {
        eprintln!("no input CSV file");
    }

    let file_path = &args[1];

    let start = std::time::SystemTime::now();
    match process_csv_file(file_path, std::io::stdout()).await {
        Ok(_) => {
            let end = std::time::SystemTime::now();
            eprintln!("elapsed: {:?}", end.duration_since(start));
        }
        Err(e) => {
            eprintln!("error: {}", e);
            exit(1);
        }
    }
}
