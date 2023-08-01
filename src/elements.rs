use greyxml::Tokens;
use std::iter::Peekable;

use crate::rss::{Channel, EncodedContent, Image, Item};

#[derive(Debug)]
pub struct Element<T: Default> {
    name: String,
    attributes: Vec<(String, String)>,
    data: T,
}

impl Element<String> {
    pub fn serialize<I>(token: Tokens, tokens: &mut Peekable<I>) -> Self
    where
        I: std::iter::Iterator<Item = Tokens>,
    {
        let Tokens::OpenNode(node) = token else {
            todo!("Implement Error Handling of Incorrect Node: {token:?}");
        };

        let mut content = String::new();
        let mut attributes = vec![];
        let mut looking_for_attributes = true;

        while let Some(token) = tokens.peek() {
            let token = token.clone();
            match token {
                Tokens::ParameterName(name) => {
                    if looking_for_attributes {
                        tokens.next();
                        if let Some(Tokens::ParameterValue(value)) = tokens.next() {
                            attributes.push((name, value));
                            continue;
                        }
                    }
                }
                Tokens::Text(text) => {
                    looking_for_attributes = false;
                    content = text;
                }
                Tokens::CloseNode(c_node) => {
                    looking_for_attributes = false;
                    if node == c_node {
                        break;
                    }
                }
                _ => {
                    looking_for_attributes = false;
                }
            }
            tokens.next();
        }

        Self {
            name: node.clone(),
            attributes,
            data: content,
        }
    }
}

impl Default for Element<String> {
    fn default() -> Self {
        Element {
            name: "".into(),
            attributes: vec![],
            data: "".into(),
        }
    }
}

impl Element<u32> {
    pub fn serialize<I>(token: Tokens, tokens: &mut Peekable<I>) -> Self
    where
        I: std::iter::Iterator<Item = Tokens>,
    {
        let Tokens::OpenNode(node) = token else {
            todo!("Implement Error Handling of Incorrect Node: {token:?}");
        };

        let mut content = 0u32;
        let mut attributes = vec![];
        let mut looking_for_attributes = true;

        while let Some(token) = tokens.peek() {
            let token = token.clone();
            match token {
                Tokens::ParameterName(name) => {
                    if looking_for_attributes {
                        tokens.next();
                        if let Some(Tokens::ParameterValue(value)) = tokens.next() {
                            attributes.push((name, value));
                            continue;
                        }
                    }
                }
                Tokens::Text(text) => {
                    looking_for_attributes = false;
                    content = text.parse().unwrap();
                }
                Tokens::CloseNode(c_node) => {
                    looking_for_attributes = false;
                    if node == c_node {
                        break;
                    }
                }
                _ => {
                    looking_for_attributes = false;
                }
            }
            tokens.next();
        }

        Self {
            name: node.clone(),
            attributes,
            data: content,
        }
    }
}

impl Element<Channel> {
    pub fn serialize<I>(token: Tokens, tokens: &mut Peekable<I>) -> Self
    where
        I: std::iter::Iterator<Item = Tokens>,
    {
        let Tokens::OpenNode(node) = token else {
            todo!("Implement Error Handling of Incorrect Node: {token:?}");
        };

        assert_eq!(node, "channel".to_string());

        let mut attributes = vec![];

        let mut channel = Channel::default();

        let mut looking_for_attributes = true;

        while let Some(token) = tokens.peek() {
            let token = token.clone();
            match token {
                Tokens::ParameterName(name) => {
                    if looking_for_attributes {
                        tokens.next();
                        if let Some(Tokens::ParameterValue(value)) = tokens.next() {
                            attributes.push((name, value));
                            continue;
                        }
                    } else {
                        tokens.next();
                    }
                }
                Tokens::OpenNode(new_node) => {
                    looking_for_attributes = false;
                    match new_node.as_str() {
                        "title" => {
                            channel.title =
                                Element::<String>::serialize(tokens.next().unwrap(), tokens);
                        }
                        "link" => {
                            channel.link =
                                Element::<String>::serialize(tokens.next().unwrap(), tokens);
                        }
                        "description" => {
                            channel.description =
                                Element::<String>::serialize(tokens.next().unwrap(), tokens);
                        }
                        "language" => {
                            channel.language =
                                Some(Element::<String>::serialize(tokens.next().unwrap(), tokens));
                        }
                        "copyright" => {
                            channel.copyright =
                                Some(Element::<String>::serialize(tokens.next().unwrap(), tokens));
                        }
                        "managingEditor" => {
                            channel.managing_editor =
                                Some(Element::<String>::serialize(tokens.next().unwrap(), tokens));
                        }
                        "webMaster" => {
                            channel.web_master =
                                Some(Element::<String>::serialize(tokens.next().unwrap(), tokens));
                        }
                        "pubDate" => {
                            channel.pub_date =
                                Some(Element::<String>::serialize(tokens.next().unwrap(), tokens));
                        }
                        "lastBuildDate" => {
                            channel.last_build_date =
                                Some(Element::<String>::serialize(tokens.next().unwrap(), tokens));
                        }
                        "category" => {
                            channel.category =
                                Some(Element::<String>::serialize(tokens.next().unwrap(), tokens));
                        }
                        "generator" => {
                            channel.generator =
                                Some(Element::<String>::serialize(tokens.next().unwrap(), tokens));
                        }
                        "docs" => {
                            channel.docs =
                                Some(Element::<String>::serialize(tokens.next().unwrap(), tokens));
                        }
                        "cloud" => {
                            channel.cloud =
                                Some(Element::<String>::serialize(tokens.next().unwrap(), tokens));
                        }
                        "rating" => {
                            channel.rating =
                                Some(Element::<String>::serialize(tokens.next().unwrap(), tokens));
                        }
                        "item" => {
                            channel
                                .items
                                .push(Element::<Item>::serialize(tokens.next().unwrap(), tokens));
                        }
                        "ttl" => {
                            channel.ttl =
                                Some(Element::<u32>::serialize(tokens.next().unwrap(), tokens));
                        }
                        "image" => {
                            channel.image =
                                Some(Element::<Image>::serialize(tokens.next().unwrap(), tokens));
                        }
                        all @ _ => {
                            println!("Unimplemented: {all:?}");
                        }
                    }
                }
                Tokens::CloseNode(close) => {
                    if close == node {
                        break;
                    }
                }
                _ => {
                    looking_for_attributes = false;
                }
            }
            tokens.next();
        }

        Self {
            name: node,
            attributes,
            data: channel,
        }
    }
}

impl Element<Item> {
    pub fn serialize<I>(token: Tokens, tokens: &mut Peekable<I>) -> Self
    where
        I: std::iter::Iterator<Item = Tokens>,
    {
        let Tokens::OpenNode(node) = token else {
            todo!("Implement Error Handling of Incorrect Node: {token:?}");
        };

        assert_eq!(node, "item".to_string());

        let mut item = Item::default();

        let mut attributes = vec![];

        let mut looking_for_attributes = true;

        while let Some(token) = tokens.peek() {
            let token = token.clone();
            match token {
                Tokens::ParameterName(name) => {
                    if looking_for_attributes {
                        tokens.next();
                        if let Some(Tokens::ParameterValue(value)) = tokens.next() {
                            attributes.push((name, value));
                            continue;
                        }
                    } else {
                        tokens.next();
                    }
                }
                Tokens::OpenNode(new_node) => {
                    looking_for_attributes = false;
                    match new_node.as_str() {
                        "title" => {
                            item.title =
                                Element::<String>::serialize(tokens.next().unwrap(), tokens)
                        }
                        "link" => {
                            item.link = Element::<String>::serialize(tokens.next().unwrap(), tokens)
                        }
                        "description" => {
                            item.description =
                                Element::<String>::serialize(tokens.next().unwrap(), tokens)
                        }
                        "author" => {
                            item.author =
                                Some(Element::<String>::serialize(tokens.next().unwrap(), tokens))
                        }
                        "category" => {
                            item.category =
                                Some(Element::<String>::serialize(tokens.next().unwrap(), tokens))
                        }
                        "enclosure" => {
                            item.enclosure =
                                Some(Element::<String>::serialize(tokens.next().unwrap(), tokens))
                        }
                        "guid" => {
                            item.guid =
                                Some(Element::<String>::serialize(tokens.next().unwrap(), tokens))
                        }
                        "pubDate" => {
                            item.pub_date =
                                Some(Element::<String>::serialize(tokens.next().unwrap(), tokens))
                        }
                        "source" => {
                            item.source =
                                Some(Element::<String>::serialize(tokens.next().unwrap(), tokens))
                        }
                        "content:encoded" => {
                            item.content_encoded = Some(Element::<EncodedContent>::serialize(
                                tokens.next().unwrap(),
                                tokens,
                            ))
                        }
                        "media" => item.media = None,
                        _ => {}
                    }
                }
                Tokens::CloseNode(close) => {
                    if close == node {
                        break;
                    }
                }
                _ => {
                    looking_for_attributes = false;
                }
            }
            tokens.next();
        }

        Self {
            name: node,
            attributes,
            data: item,
        }
    }
}

impl Element<EncodedContent> {
    pub fn serialize<I>(token: Tokens, tokens: &mut Peekable<I>) -> Self
    where
        I: std::iter::Iterator<Item = Tokens>,
    {
        let Tokens::OpenNode(node) = token else {
            todo!("Implement Error Handling of Incorrect Node: {token:?}");
        };

        assert_eq!(node, "content:encoded".to_string());

        let mut looking_for_attributes = true;

        let mut attributes = vec![];
        let mut elements = vec![];
        let mut content = String::new();

        while let Some(token) = tokens.peek() {
            let token = token.clone();
            match token {
                Tokens::ParameterName(name) => {
                    if looking_for_attributes {
                        tokens.next();
                        if let Some(Tokens::ParameterValue(value)) = tokens.next() {
                            attributes.push((name, value));
                            continue;
                        }
                    } else {
                        tokens.next();
                    }
                }
                Tokens::OpenNode(_) => {
                    looking_for_attributes = false;
                    elements.push(Element::<String>::serialize(tokens.next().unwrap(), tokens));
                }
                Tokens::Text(text) => content = text,
                Tokens::CloseNode(close) => {
                    looking_for_attributes = false;
                    if node == close {
                        break;
                    }
                }
                _ => {
                    looking_for_attributes = false;
                }
            }
            tokens.next();
        }

        Self {
            name: node,
            attributes,
            data: EncodedContent(elements, content),
        }
    }
}

impl Element<Image> {
    pub fn serialize<I>(token: Tokens, tokens: &mut Peekable<I>) -> Self
    where
        I: std::iter::Iterator<Item = Tokens>,
    {
        let Tokens::OpenNode(node) = token else {
            todo!("Implement Error Handling of Incorrect Node: {token:?}");
        };

        assert_eq!(node, "image".to_string());

        let mut looking_for_attributes = true;

        let mut attributes = vec![];

        let mut image = Image::default();

        while let Some(token) = tokens.peek() {
            let token = token.clone();
            match token {
                Tokens::ParameterName(name) => {
                    if looking_for_attributes {
                        tokens.next();
                        if let Some(Tokens::ParameterValue(value)) = tokens.next() {
                            attributes.push((name, value));
                            continue;
                        }
                    } else {
                        tokens.next();
                    }
                }
                Tokens::OpenNode(new_node) => match new_node.as_str() {
                    "url" => {
                        image.url = Element::<String>::serialize(tokens.next().unwrap(), tokens)
                    }
                    "title" => {
                        image.title = Element::<String>::serialize(tokens.next().unwrap(), tokens)
                    }
                    "link" => {
                        image.link = Element::<String>::serialize(tokens.next().unwrap(), tokens)
                    }
                    "description" => {
                        image.description =
                            Some(Element::<String>::serialize(tokens.next().unwrap(), tokens))
                    }
                    "width" => {
                        image.width =
                            Some(Element::<u32>::serialize(tokens.next().unwrap(), tokens))
                    }
                    "height" => {
                        image.height =
                            Some(Element::<u32>::serialize(tokens.next().unwrap(), tokens))
                    }
                    other @ _ => {
                        println!("Unexpected/Unimplemented Image Element: {other:?}")
                    }
                },
                Tokens::CloseNode(close) => {
                    looking_for_attributes = false;
                    if node == close {
                        break;
                    }
                }
                _ => {
                    looking_for_attributes = false;
                }
            }
            tokens.next();
        }

        Self {
            name: node,
            attributes,
            data: image,
        }
    }
}
