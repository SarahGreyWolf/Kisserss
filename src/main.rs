use std::env;
use std::io::Read;
use std::path::PathBuf;
use std::{error::Error, fs::File};

use greyxml::{lex, tokenize};

fn main() -> Result<(), Box<dyn Error>> {
    let mut input = String::new();
    let mut args = env::args();
    // Skip process name
    args.next();
    let Some(path_string) = args.next() else {
        panic!("No path given");
    };
    let path = PathBuf::from(path_string);
    let mut file = File::open(path)?;
    file.read_to_string(&mut input)?;
    let lexed = lex(input)?;
    let mut iter = lexed.into_iter();
    let tokens = tokenize(&mut iter)?;
    println!("{:?}", tokens);

    Ok(())
}
