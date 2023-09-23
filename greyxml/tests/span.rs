use greyxml::{lex, tokenize, Lexicals};

type TestResult<T> = Result<T, Box<dyn std::error::Error>>;

fn test_span(
    lexical: &Lexicals,
    expected_row: usize,
    expected_column: usize,
    expected_length: usize,
) {
    match lexical {
        Lexicals::OpenArrow(span)
        | Lexicals::CloseArrow(span)
        | Lexicals::ImmediateClose(span)
        | Lexicals::CloseFSlash(span)
        | Lexicals::LeftSquareBracket(span)
        | Lexicals::RightSquareBracket(span)
        | Lexicals::Bang(span)
        | Lexicals::Equals(span)
        | Lexicals::DoubleQuote(span)
        | Lexicals::Question(span) => {
            assert_eq!(span.row, expected_row);
            assert_eq!(span.column, expected_column);
            assert_eq!(span.length, expected_length);
        }
        Lexicals::Text(_, span) => {
            assert_eq!(span.row, expected_row);
            assert_eq!(span.column, expected_column);
            assert_eq!(span.length, expected_length);
        }
    }
}

#[test]
fn test_simple_span() -> TestResult<()> {
    let xml = r#"<html></html>"#;
    let lexed = lex(xml)?;

    test_span(&lexed[0], 1, 1, 1);
    test_span(&lexed[1], 1, 2, 4);
    test_span(&lexed[2], 1, 6, 1);
    test_span(&lexed[3], 1, 7, 1);
    test_span(&lexed[4], 1, 8, 1);
    test_span(&lexed[5], 1, 9, 4);
    test_span(&lexed[6], 1, 13, 1);

    Ok(())
}

#[test]
fn test_multiline_span() -> TestResult<()> {
    let xml = r#"<html>
    <div>
        This is
        Some Multiline
        Text
    </div>
</html>"#;
    let lexed = lex(xml)?;

    test_span(&lexed[0], 1, 1, 1);
    test_span(&lexed[1], 1, 2, 4);
    test_span(&lexed[2], 1, 6, 1);
    test_span(&lexed[3], 2, 1, 4);
    test_span(&lexed[4], 2, 5, 1);
    test_span(&lexed[5], 2, 6, 3);
    test_span(&lexed[6], 2, 9, 1);
    test_span(&lexed[7], 2, 1, 56);
    test_span(&lexed[8], 3, 1, 1);
    test_span(&lexed[9], 3, 2, 1);
    test_span(&lexed[10], 3, 3, 3);
    test_span(&lexed[11], 3, 6, 1);
    test_span(&lexed[12], 4, 1, 1);
    test_span(&lexed[13], 4, 2, 1);
    test_span(&lexed[14], 4, 3, 4);
    test_span(&lexed[15], 4, 7, 1);

    Ok(())
}