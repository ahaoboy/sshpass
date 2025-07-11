use sshpass::{Error, parse_options};
use std::{env, process};

fn main() -> Result<(), Error> {
    let argv: Vec<String> = env::args().collect();
    let argc = argv.len();

    match parse_options(argc, &argv) {
        Ok(option) => {
            sshpass::sshpass(&option)?;
        }
        Err(_) => {
            process::exit(-1);
        }
    }

    Ok(())
}
