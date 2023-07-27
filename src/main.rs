use std::{fs::File, error::Error};
use std::io::Read;

mod parser;

fn main() -> Result<(), Box<dyn Error>> {
    let mut input = String::new();
    let mut file = File::open("./kisserss.rss")?;
    file.read_to_string(&mut input)?;
    let lexed = parser::lex(input)?;
    let mut iter = lexed.into_iter();
    let tokens = parser::tokenize(&mut iter)?;
    println!("{:?}", tokens);

    Ok(())
}
