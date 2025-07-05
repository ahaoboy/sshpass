use sshpass::{Error, parse_options};
use std::env;

fn main() -> Result<(), Error> {
    let argv: Vec<String> = env::args().collect();
    let argc = argv.len();
    let opt = parse_options(argc, &argv)?;
    let pwd = opt.get_password()?;
    println!("pwd: {pwd}");
    println!("opt: {opt:?}");
    Ok(())
}
