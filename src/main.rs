use std::env;
use std::io::Read;
use std::path::PathBuf;
use std::{error::Error, fs::File};

mod elements;
mod rss;

fn main() -> Result<(), Box<dyn Error>> {
    let mut input = String::new();
    let mut args = env::args();
    args.next();
    let Some(path_string) = args.next() else {
        panic!("No path given");
    };
    let path = PathBuf::from(path_string);
    let mut file = File::open(path)?;
    file.read_to_string(&mut input)?;
    let feed = rss::Feed::serialize(&input)?;
    dbg!(feed);
    Ok(())
}
