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
    Text(String),
}

pub fn lex(stream: String) -> Result<Vec<Lexicals>, Box<dyn Error>> {
    let mut lexed = vec![];
    let mut peekable = stream.chars().into_iter().peekable();
    let mut in_block = false;
    let mut in_q_block = false;
    let mut in_simple_block = false;
    let mut is_defining_node_name = false;
    let mut in_quote = false;
    let mut temp_string = String::new();
    while let Some(c) = peekable.peek() {
        let c = *c;
        peekable.next();
        if is_defining_node_name && !temp_string.is_empty() {}
        match c {
            '<' => {
                if peekable.peek() != Some(&'!') && !in_q_block {}
                in_block = true;
                if peekable.peek() == Some(&'?') {
                    in_simple_block = true;
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
                is_defining_node_name = true;
            }
            '?' => {
                if peekable.peek() == Some(&'>') {
                    in_simple_block = false;
                    lexed.push(Lexicals::ImmediateClose);
                    peekable.next();
                } else {
                    lexed.push(Lexicals::Question);
                }
            }
            '>' => {
                if in_block {
                    if !temp_string.is_empty() {
                        lexed.push(Lexicals::Text(temp_string));
                        temp_string = String::new();
                    }
                    in_block = false;
                }
                lexed.push(Lexicals::CloseArrow);
                is_defining_node_name = false;
            }
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
            }
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
            }
            '!' => {
                lexed.push(Lexicals::Bang);
            }
            '=' => {
                if in_quote {
                    temp_string.push(c);
                    continue;
                }
                if (in_block || in_q_block || in_simple_block) && !temp_string.is_empty() {
                    lexed.push(Lexicals::Text(temp_string));
                    temp_string = String::new();
                    lexed.push(Lexicals::Equals);
                } else {
                    temp_string.push(c);
                }
            }
            '"' => {
                in_quote = !in_quote;
                if (in_block || in_simple_block) && !temp_string.is_empty() {
                    lexed.push(Lexicals::Text(temp_string));
                    temp_string = String::new();
                }
                lexed.push(Lexicals::DoubleQuote);
            }
            '/' => {
                if !in_quote {
                    lexed.push(Lexicals::CloseFSlash);
                } else {
                    temp_string.push(c);
                }
            }
            /*
            '?' => {
                lexed.push(Lexicals::Question);
                in_q_block = !in_q_block;
            },
            */
            _ => {
                if in_quote {
                    temp_string.push(c);
                    continue;
                }
                if c == ' ' {
                    if !is_defining_node_name {
                        temp_string.push(c);
                    } else {
                        if !temp_string.is_empty() {
                            lexed.push(Lexicals::Text(temp_string));
                            temp_string = String::new();
                        }
                        is_defining_node_name = false;
                    }
                } else {
                    temp_string.push(c);
                }
            }
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
    // A single line node, such as the xml version info
    SimpleNode(String),
    OpenNode(String),
    ParameterName(String),
    ParameterValue(String),
    CloseNode(String),
    Text(String),
}

pub fn tokenize<T: Sized>(lexed: &mut T) -> Result<Vec<Tokens>, Box<dyn Error>>
where
    T: Iterator<Item = Lexicals>,
{
    let mut tokens = vec![];
    let mut peekable = lexed.peekable();
    let mut in_simple_node = false;
    let mut in_node = false;
    let mut in_close_node = false;
    let mut is_param_value = false;
    let mut node_names: Vec<String> = vec![];
    while let Some(lx) = peekable.peek() {
        let lex = lx.to_owned();
        peekable.next();
        let peek = peekable.peek();
        match lex {
            Lexicals::OpenArrow => {
                if peekable.peek() == Some(&Lexicals::Question) {
                    in_simple_node = true;
                } else if peekable.peek() != Some(&Lexicals::CloseFSlash) {
                    in_node = true;
                }
            }
            Lexicals::CloseArrow => {
                in_close_node = false;
                in_node = false;
                in_simple_node = false;
            }
            Lexicals::ImmediateClose => {
                in_simple_node = false;
                node_names.pop();
            }
            Lexicals::CloseFSlash => {
                if in_node {
                    in_node = false;
                    if let Some(name) = node_names.pop() {
                        tokens.push(Tokens::CloseNode(name));
                    }
                } else {
                    in_close_node = true;
                }
            }
            Lexicals::LeftSquareBracket => {}
            Lexicals::RightSquareBracket => {
                if peekable.peek() == Some(&Lexicals::CloseArrow) {
                    peekable.next();
                }
            }
            Lexicals::Bang => {
                if peekable.peek() != Some(&Lexicals::LeftSquareBracket) {
                    in_simple_node = true;
                }
                if in_node {
                    in_node = false;
                }
            }
            Lexicals::Equals => {
                if in_node || in_simple_node {
                    is_param_value = true;
                }
            }
            Lexicals::DoubleQuote => {}
            Lexicals::Question => {}
            Lexicals::Text(text) => {
                let trimmed = text.trim();
                if in_node {
                    if is_param_value {
                        tokens.push(Tokens::ParameterValue(trimmed.into()));
                        is_param_value = false;
                        continue;
                    }
                    if peekable.peek() == Some(&Lexicals::Equals) {
                        tokens.push(Tokens::ParameterName(trimmed.into()));
                        continue;
                    }
                    let text = text.trim_start();
                    node_names.push(text.into());
                    tokens.push(Tokens::OpenNode(text.into()));
                    continue;
                }
                if in_close_node {
                    tokens.push(Tokens::CloseNode(trimmed.into()));
                    node_names.pop();
                    continue;
                }
                if in_simple_node {
                    if is_param_value
                        || (!node_names.is_empty() && peekable.peek() != Some(&Lexicals::Equals))
                    {
                        tokens.push(Tokens::ParameterValue(trimmed.into()));
                        is_param_value = false;
                        continue;
                    }
                    if peekable.peek() == Some(&Lexicals::Equals) {
                        tokens.push(Tokens::ParameterName(trimmed.into()));
                        continue;
                    }
                    tokens.push(Tokens::SimpleNode(trimmed.into()));
                    node_names.push(trimmed.into());
                    continue;
                }
                if trimmed.is_empty() {
                    continue;
                }
                tokens.push(Tokens::Text(trimmed.into()));
            }
        }
    }
    Ok(tokens)
}
