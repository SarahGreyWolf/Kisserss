use std::env;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::PathBuf;
use std::{error::Error, fs::File};

use greyxml::{lex, tokenize};
use reqwest::blocking::get;

mod elements;
mod rss;

fn main() -> Result<(), Box<dyn Error>> {
    let mut input = String::new();
    let mut args = env::args();
    args.next();
    let Some(path_string) = args.next() else {
        panic!("No path given");
    };
    if path_string.starts_with("http://") || path_string.starts_with("https://") {
        // FIXME: Handle Errors
        input = get_web_feed(&path_string)?;
    } else {
        let path = PathBuf::from(path_string);
        let mut file = File::open(path)?;
        file.read_to_string(&mut input)?;
    }
    let feed = rss::Feed::serialize(&input)?;
    dbg!(feed);
    Ok(())
}

fn get_web_feed(source: &str) -> Result<String, Box<dyn Error>> {
    let body = get(source)?.text()?;
    Ok(body)
}
