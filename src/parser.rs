use std::error::Error;

#[derive(Clone, Debug, PartialEq)]
pub enum Lexicals {
    OpenArrow,
    CloseArrow,
    ImmediateClose,
    CloseFSlash,
    LeftSquareBracket,
    RightSquareBracket,
    Bang,
    Equals,
    DoubleQuote,
    Question,
    Text(String)
}

pub fn lex(stream: String) -> Result<Vec<Lexicals>, Box<dyn Error>> {
    let mut lexed = vec![];
    let mut peekable = stream.chars().into_iter().peekable();
    let mut in_block = false;
    let mut in_q_block = false;
    let mut temp_string = String::new();
    while let Some(c) = peekable.peek() {
        let c = *c;
        peekable.next();
        match c {
            '<' => {
                if peekable.peek() != Some(&'!') && !in_q_block {
                    in_block = true;
                }
                if peekable.peek() == Some(&'/') {
                    if !temp_string.is_empty() {
                        lexed.push(Lexicals::Text(temp_string));
                        temp_string = String::new();
                    }
                    lexed.push(Lexicals::OpenArrow);
                    lexed.push(Lexicals::CloseFSlash);
                    peekable.next();
                } else {
                    lexed.push(Lexicals::OpenArrow);
                }
            },
            '?' => {
                if peekable.peek() == Some(&'>') {
                    lexed.push(Lexicals::ImmediateClose);
                }
            },
            '>' => {
                if in_block {
                    if !temp_string.is_empty() {
                        lexed.push(Lexicals::Text(temp_string));
                        temp_string = String::new();
                    }
                    in_block = false;
                }
                if in_q_block {
                    in_q_block = false;
                }
                lexed.push(Lexicals::CloseArrow);
            },
            '[' => {
                if in_block {
                    in_block = false;
                    in_q_block = true;
                }
                if !temp_string.is_empty() {
                    lexed.push(Lexicals::Text(temp_string));
                    temp_string = String::new();
                }
                lexed.push(Lexicals::LeftSquareBracket);
            },
            ']' => {
                if in_q_block {
                    in_q_block = false;
                    in_block = true;
                }
                if !temp_string.is_empty() {
                    lexed.push(Lexicals::Text(temp_string));
                    temp_string = String::new();
                }
                lexed.push(Lexicals::RightSquareBracket);
            },
            '!' => {
                lexed.push(Lexicals::Bang);
                in_block = false;
                in_q_block = true;
            },
            '=' => {
                if in_q_block && !temp_string.is_empty() {
                    lexed.push(Lexicals::Text(temp_string));
                    temp_string = String::new();
                    lexed.push(Lexicals::Equals);
                } else {
                    temp_string.push(c);
                }
            },
            '"' => {
                if in_block && !temp_string.is_empty() {
                    lexed.push(Lexicals::Text(temp_string));
                    temp_string = String::new();
                }
                in_block = !in_block;
                lexed.push(Lexicals::DoubleQuote);
            },
            /*
            '?' => {
                lexed.push(Lexicals::Question);
                in_q_block = !in_q_block;
            },
            */
            _ => {
                if temp_string.len() >= 4 {
                    if temp_string[0..4].contains("rss") {
                        in_block = false;
                        in_q_block = true;
                    }
                }
                if c == ' ' {
                    if !in_q_block {
                        temp_string.push(c);
                    } else {
                        if !temp_string.is_empty() {
                            lexed.push(Lexicals::Text(temp_string));
                            temp_string = String::new();
                        }
                    }
                } else {
                    temp_string.push(c);
                }
            },
        }
    }
    Ok(lexed)
}

#[derive(Debug)]
pub enum Ast {
    // Takes an ElementNode
    DescriptorNode(Box<Tokens>),
    // Has a name, a series of attributes, and any child Nodes
    ElementNode(String, Vec<Box<Tokens>>, Vec<Box<Tokens>>),
    // An attribute is the attribute name and a TextNode
    Attribute(String, Box<Tokens>),
    TextNode(String),
}

#[derive(Debug)]
pub enum Tokens {
    // The vector contains ParameterNames
    OpenNode(String),
    ParameterName(String),
    ParameterValue(String),
    CloseNode(String),
    Text(String)
}

pub fn tokenize<T: Sized>(lexed: &mut T) -> Result<Vec<Tokens>, Box<dyn Error>>
where T: Iterator<Item = Lexicals> {
    let mut tokens = vec![];
    let mut peekable = lexed.peekable();
    let mut in_node = false;
    let mut in_close_node = false;
    let mut is_param_value = false;
    let mut node_name = String::new();
    while let Some(lx) = peekable.peek() {
        let lex = lx.to_owned();
        peekable.next();

        match lex {
            Lexicals::OpenArrow => {
                if peekable.peek() != Some(&Lexicals::CloseFSlash) {
                    in_node = true;
                }
            },
            Lexicals::CloseArrow => {
                in_close_node = false;
                in_node = false;
            },
            Lexicals::ImmediateClose => {
//                tokens.push(Tokens::CloseNode(node_name));
            },
            Lexicals::CloseFSlash => {
                in_close_node = true;
            },
            Lexicals::LeftSquareBracket => {},
            Lexicals::RightSquareBracket => {},
            Lexicals::Bang => {},
            Lexicals::Equals => {
                if in_node {
                    is_param_value = true;
                }
            },
            Lexicals::DoubleQuote => {
            },
            Lexicals::Question => {},
            Lexicals::Text(text) => {
                if in_node {
                    if is_param_value {
                        let text = text.trim();
                        tokens.push(Tokens::ParameterValue(text.into()));
                        is_param_value = false;
                        continue;
                    }
                    if peekable.peek() == Some(&Lexicals::Equals) {
                        let text = text.trim();
                        tokens.push(Tokens::ParameterName(text.into()));
                        continue;
                    }
                    let text = text.trim_start();
                    node_name = text.into();
                    tokens.push(Tokens::OpenNode(text.into()));
                    continue;
                }
                if in_close_node {
                    let text = text.trim();
                    tokens.push(Tokens::CloseNode(text.into()));
                    continue;
                }
                let test_text = text.trim();
                if test_text.is_empty() {
                    continue;
                }
                tokens.push(Tokens::Text(text.into()));
            }
        }
    }
    Ok(tokens)
}
