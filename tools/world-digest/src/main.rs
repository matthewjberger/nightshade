mod commands;
mod data;
mod queries;
mod resources;

use commands::report_the_digest;
use queries::read_the_run;

fn main() {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    match read_the_run(&arguments) {
        Ok(run) => report_the_digest(&run),
        Err(trouble) => {
            eprintln!("{trouble}");
            std::process::exit(2);
        }
    }
}
