use std::error::Error;

#[derive(Clone, Debug, PartialEq)]
pub enum Lexicals {
    OpenArrow(Span),
    CloseArrow(Span),
    ImmediateClose(Span),
    CloseFSlash(Span),
    LeftSquareBracket(Span),
    RightSquareBracket(Span),
    Bang(Span),
    Equals(Span),
    DoubleQuote(Span),
    Question(Span),
    Text(String, Span),
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Span {
    pub row: usize,
    pub column: usize,
    pub length: usize,
}

pub fn lex(stream: String) -> Result<Vec<Lexicals>, Box<dyn Error>> {
    let mut lexed = vec![];
    let mut peekable = stream.chars().peekable();
    let mut in_block = false;
    let mut in_q_block = false;
    let mut in_simple_block = false;
    let mut is_defining_node_name = false;
    let mut in_quote = false;
    let mut temp_string = String::new();
    let mut current_span = Span::default();
    while let Some(c) = peekable.peek() {
        current_span.column += 1;
        let c = *c;
        peekable.next();
        match c {
            '<' => {
                in_block = true;
                if peekable.peek() == Some(&'?') {
                    in_simple_block = true;
                }
                if peekable.peek() == Some(&'/') {
                    if !temp_string.is_empty() {
                        current_span.length -= 1;
                        if current_span.column >= current_span.length {
                            current_span.column -= current_span.length;
                        }
                        lexed.push(Lexicals::Text(temp_string, current_span.clone()));
                        current_span.column += current_span.length;
                        current_span.length = 1;
                        temp_string = String::new();
                    }
                    lexed.push(Lexicals::OpenArrow(current_span.clone()));
                    current_span.column += 1;
                    current_span.length = 1;
                    lexed.push(Lexicals::CloseFSlash(current_span.clone()));
                    current_span.length = 0;
                    peekable.next();
                } else {
                    lexed.push(Lexicals::OpenArrow(current_span.clone()));
                    current_span.length = 0;
                }
                is_defining_node_name = true;
            }
            '?' => {
                if peekable.peek() == Some(&'>') {
                    in_simple_block = false;
                    lexed.push(Lexicals::ImmediateClose(current_span.clone()));
                    current_span.length = 0;
                    peekable.next();
                } else {
                    lexed.push(Lexicals::Question(current_span.clone()));
                    current_span.length = 0;
                }
            }
            '>' => {
                if in_block {
                    if !temp_string.is_empty() {
                        current_span.length -= 1;
                        if current_span.column >= current_span.length {
                            current_span.column -= current_span.length;
                        }
                        lexed.push(Lexicals::Text(temp_string, current_span.clone()));
                        current_span.column += current_span.length;
                        current_span.length = 1;
                        temp_string = String::new();
                    }
                    in_block = false;
                }
                lexed.push(Lexicals::CloseArrow(current_span.clone()));
                current_span.length = 0;
                is_defining_node_name = false;
            }
            '[' => {
                if in_block {
                    in_block = false;
                    in_q_block = true;
                }
                if !temp_string.is_empty() {
                    current_span.length -= 1;
                    current_span.column -= current_span.length;
                    lexed.push(Lexicals::Text(temp_string, current_span.clone()));
                    current_span.column += current_span.length;
                    current_span.length = 1;
                    temp_string = String::new();
                }
                lexed.push(Lexicals::LeftSquareBracket(current_span.clone()));
                current_span.length = 0;
            }
            ']' => {
                if in_q_block {
                    in_q_block = false;
                    in_block = true;
                }
                if !temp_string.is_empty() {
                    current_span.length -= 1;
                    current_span.column -= current_span.length;
                    lexed.push(Lexicals::Text(temp_string, current_span.clone()));
                    current_span.column += current_span.length;
                    current_span.length = 1;
                    temp_string = String::new();
                }
                lexed.push(Lexicals::RightSquareBracket(current_span.clone()));
                current_span.length = 0;
            }
            '!' => {
                lexed.push(Lexicals::Bang(current_span.clone()));
                current_span.length = 0;
            }
            '=' => {
                if in_quote {
                    temp_string.push(c);
                    continue;
                }
                if (in_block || in_q_block || in_simple_block) && !temp_string.is_empty() {
                    current_span.length -= 1;
                    current_span.column -= current_span.length;
                    lexed.push(Lexicals::Text(temp_string, current_span.clone()));
                    current_span.column += current_span.length;
                    temp_string = String::new();
                    current_span.length = 1;
                    lexed.push(Lexicals::Equals(current_span.clone()));
                    current_span.length = 0;
                } else {
                    temp_string.push(c);
                }
            }
            '"' => {
                in_quote = !in_quote;
                if (in_block || in_simple_block) && !temp_string.is_empty() {
                    current_span.length -= 1;
                    current_span.column -= current_span.length;
                    lexed.push(Lexicals::Text(temp_string, current_span.clone()));
                    current_span.column += current_span.length;
                    temp_string = String::new();
                    current_span.length = 1;
                }
                lexed.push(Lexicals::DoubleQuote(current_span.clone()));
                current_span.length = 0;
            }
            '/' => {
                if !in_quote && (in_block || in_simple_block) {
                    lexed.push(Lexicals::CloseFSlash(current_span.clone()));
                    current_span.length = 0;
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
            '\n' => {
                current_span.row += 1;
                current_span.column = 0;
            }
            _ => {
                if in_quote {
                    temp_string.push(c);
                    current_span.length += 1;
                    continue;
                }
                if c == ' ' {
                    if !is_defining_node_name {
                        temp_string.push(c);
                    } else {
                        if !temp_string.is_empty() {
                            current_span.length -= 1;
                            if current_span.column >= current_span.length {
                                current_span.column -= current_span.length;
                            }
                            lexed.push(Lexicals::Text(temp_string, current_span.clone()));
                            current_span.column += current_span.length;
                            temp_string = String::new();
                            current_span.length = 0;
                        }
                        is_defining_node_name = false;
                    }
                } else {
                    temp_string.push(c);
                }
            }
        }
        current_span.length += 1;
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

#[derive(Clone, Debug, PartialEq)]
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
    let mut current_node = String::new();
    while let Some(lx) = peekable.peek() {
        let lex = lx.to_owned();
        peekable.next();
        match lex {
            Lexicals::OpenArrow(_) => {
                if let Some(lex) = peekable.peek() {
                    match lex {
                        Lexicals::Question(_) => in_simple_node = true,
                        Lexicals::CloseFSlash(_) => {}
                        _ => {
                            in_node = true;
                        }
                    }
                }
            }
            Lexicals::CloseArrow(_) => {
                current_node = String::new();
                in_close_node = false;
                in_node = false;
                in_simple_node = false;
            }
            Lexicals::ImmediateClose(_) => {
                in_simple_node = false;
                node_names.pop();
            }
            Lexicals::CloseFSlash(_) => {
                if in_node {
                    in_node = false;
                    if let Some(name) = node_names.pop() {
                        current_node = name.clone();
                        tokens.push(Tokens::CloseNode(name));
                    } else {
                        current_node = String::new();
                    }
                } else {
                    in_close_node = true;
                }
            }
            Lexicals::LeftSquareBracket(_) => {}
            Lexicals::RightSquareBracket(_) => {
                if let Some(Lexicals::CloseArrow(_)) = peekable.peek() {
                    peekable.next();
                }
            }
            Lexicals::Bang(_) => {
                if let Some(Lexicals::LeftSquareBracket(_)) = peekable.peek() {
                } else {
                    in_simple_node = true;
                }
                if in_node {
                    in_node = false;
                    current_node = String::new();
                }
            }
            Lexicals::Equals(_) => {
                if in_node || in_simple_node {
                    is_param_value = true;
                }
            }
            Lexicals::DoubleQuote(_) => {}
            Lexicals::Question(_) => {}
            Lexicals::Text(text, _) => {
                let trimmed = text.trim();
                if in_node {
                    if let Some(lex) = peekable.peek() {
                        match lex {
                            Lexicals::Equals(_) => {
                                tokens.push(Tokens::ParameterName(trimmed.into()));
                                continue;
                            }
                            _ => {
                                if is_param_value {
                                    tokens.push(Tokens::ParameterValue(trimmed.into()));
                                    is_param_value = false;
                                    continue;
                                }
                                if let Some(last) = node_names.last() {
                                    if *last == current_node && *last != trimmed {
                                        tokens.push(Tokens::ParameterName(trimmed.into()));
                                        tokens.push(Tokens::ParameterValue("true".into()));
                                        is_param_value = false;
                                        continue;
                                    }
                                }
                            }
                        }
                    }
                    node_names.push(trimmed.into());
                    current_node = trimmed.into();
                    tokens.push(Tokens::OpenNode(trimmed.into()));
                    continue;
                }
                if in_close_node {
                    tokens.push(Tokens::CloseNode(trimmed.into()));
                    if let Some(popped) = node_names.pop() {
                        current_node = popped;
                    } else {
                        current_node = String::new();
                    }
                    continue;
                }
                if in_simple_node {
                    if let Some(lex) = peekable.peek() {
                        match lex {
                            Lexicals::Equals(_) => {
                                tokens.push(Tokens::ParameterName(trimmed.into()));
                                continue;
                            }
                            _ => {
                                if is_param_value {
                                    tokens.push(Tokens::ParameterValue(trimmed.into()));
                                    is_param_value = false;
                                    continue;
                                }
                                if let Some(last) = node_names.last() {
                                    if *last == current_node {
                                        tokens.push(Tokens::ParameterName(trimmed.into()));
                                        tokens.push(Tokens::ParameterValue("true".into()));
                                        is_param_value = false;
                                        continue;
                                    }
                                }
                            }
                        }
                    }
                    current_node = trimmed.into();
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
