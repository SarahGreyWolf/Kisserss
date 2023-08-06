use greyxml::{lex, tokenize};
use std::fs::File;
use std::io::Read;

type TestResult<T> = Result<T, Box<dyn std::error::Error>>;

fn test_against_file(test_path: &str, test_output_path: &str) -> TestResult<()> {
    let mut test_file = File::open(test_path)?;
    let mut test_output = File::open(test_output_path)?;
    let mut test_data = String::new();
    let mut output = String::new();
    test_file.read_to_string(&mut test_data)?;
    test_output.read_to_string(&mut output)?;

    let lexed = lex(test_data)?;
    let mut iter = lexed.into_iter();
    let tokens = tokenize(&mut iter)?;

    assert_eq!(format!("{:?}", tokens), output);

    Ok(())
}

#[test]
fn rss() -> TestResult<()> {
    test_against_file("./tests/test.rss", "./tests/test.rss.output")?;
    Ok(())
}

#[test]
fn repo_rss() -> TestResult<()> {
    test_against_file("./tests/kisserss.rss", "./tests/kisserss.rss.output")?;
    Ok(())
}

#[test]
fn html() -> TestResult<()> {
    test_against_file("./tests/example.html", "./tests/example.html.output")?;
    Ok(())
}
