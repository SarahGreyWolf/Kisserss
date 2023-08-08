use greyxml::Tokens;
use std::iter::Peekable;

use crate::atom;
use crate::rss::{Channel, EncodedContent, Image, Item};

#[derive(Debug, Default)]
pub struct Element<T: Default> {
    pub name: String,
    pub attributes: Vec<(String, String)>,
    pub data: T,
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

// RSS

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
                            //println!("Unimplemented: {all:?}");
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
                                Some(Element::<String>::serialize(tokens.next().unwrap(), tokens))
                        }
                        "link" => {
                            item.link =
                                Some(Element::<String>::serialize(tokens.next().unwrap(), tokens))
                        }
                        "description" => {
                            item.description =
                                Some(Element::<String>::serialize(tokens.next().unwrap(), tokens))
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
                Tokens::Text(text) => {
                    elements.push(Element {
                        name: "p".into(),
                        attributes: vec![],
                        data: text,
                    });
                }
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
                        //println!("Unexpected/Unimplemented Image Element: {other:?}")
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

// ATOM
/* TEMPLATE
impl Element<atom::<INSERT>> {
    pub fn serialize<I>(token: Tokens, tokens: &mut Peekable<I>) -> Self
    where
        I: std::iter::Iterator<Item = Tokens>,
    {
        let Tokens::OpenNode(node) = token else {
            todo!("Implement Error Handling of Incorrect Node: {token:?}");
        };

        let mut attributes = vec![];

        let mut <INSERT> = atom::<INSERT>::default();

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
            data: <INSERT>,
        }
    }
}
*/

impl Element<atom::Person> {
    pub fn serialize<I>(token: Tokens, tokens: &mut Peekable<I>) -> Self
    where
        I: std::iter::Iterator<Item = Tokens>,
    {
        let Tokens::OpenNode(node) = token else {
            todo!("Implement Error Handling of Incorrect Node: {token:?}");
        };

        let mut attributes = vec![];

        let mut person = atom::Person::default();

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
                        "name" => {
                            person.name =
                                Element::<String>::serialize(tokens.next().unwrap(), tokens);
                        }
                        "uri" => {
                            person.uri =
                                Some(Element::<String>::serialize(tokens.next().unwrap(), tokens));
                        }
                        "email" => {
                            person.email =
                                Some(Element::<String>::serialize(tokens.next().unwrap(), tokens));
                        }
                        all @ _ => {
                            //println!("Unimplemented: {all:?}");
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
            data: person,
        }
    }
}

impl Element<atom::Category> {
    pub fn serialize<I>(token: Tokens, tokens: &mut Peekable<I>) -> Self
    where
        I: std::iter::Iterator<Item = Tokens>,
    {
        let Tokens::OpenNode(node) = token else {
            todo!("Implement Error Handling of Incorrect Node: {token:?}");
        };

        let mut attributes = vec![];

        let mut category = atom::Category::default();

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
                        all @ _ => {
                            //println!("Unimplemented: {all:?}");
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
            data: category,
        }
    }
}

impl Element<atom::Link> {
    pub fn serialize<I>(token: Tokens, tokens: &mut Peekable<I>) -> Self
    where
        I: std::iter::Iterator<Item = Tokens>,
    {
        let Tokens::OpenNode(node) = token else {
            todo!("Implement Error Handling of Incorrect Node: {token:?}");
        };

        let mut attributes = vec![];

        let mut link = atom::Link::default();

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
                        all @ _ => {
                            //println!("Unimplemented: {all:?}");
                        }
                    }
                }
                Tokens::Text(text) => {
                    looking_for_attributes = false;
                    link.0 = Some(text);
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
            data: link,
        }
    }
}

impl Element<Option<atom::Entry>> {
    pub fn serialize<I>(token: Tokens, tokens: &mut Peekable<I>) -> Self
    where
        I: std::iter::Iterator<Item = Tokens>,
    {
        let Tokens::OpenNode(node) = token else {
            todo!("Implement Error Handling of Incorrect Node: {token:?}");
        };

        let mut attributes = vec![];

        let mut entry = atom::Entry::default();

        let mut looking_for_attributes = true;

        let mut authors = vec![];
        let mut categories = vec![];
        let mut contributors = vec![];
        let mut links = vec![];

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
                        "author" => authors.push(Element::<atom::Person>::serialize(
                            tokens.next().unwrap(),
                            tokens,
                        )),
                        "category" => categories.push(Element::<atom::Category>::serialize(
                            tokens.next().unwrap(),
                            tokens,
                        )),
                        "content" => {
                            entry.content =
                                Element::<String>::serialize(tokens.next().unwrap(), tokens)
                        }
                        "contributor" => contributors.push(Element::<atom::Person>::serialize(
                            tokens.next().unwrap(),
                            tokens,
                        )),
                        "id" => {
                            entry.id = Element::<String>::serialize(tokens.next().unwrap(), tokens)
                        }
                        "link" => links.push(Element::<atom::Link>::serialize(
                            tokens.next().unwrap(),
                            tokens,
                        )),
                        "published" => {
                            entry.published =
                                Some(Element::<String>::serialize(tokens.next().unwrap(), tokens))
                        }
                        "rights" => {
                            entry.rights =
                                Some(Element::<String>::serialize(tokens.next().unwrap(), tokens))
                        }
                        "summary" => {
                            entry.summary =
                                Some(Element::<String>::serialize(tokens.next().unwrap(), tokens))
                        }
                        "title" => {
                            entry.title =
                                Element::<String>::serialize(tokens.next().unwrap(), tokens)
                        }
                        "updated" => {
                            entry.updated =
                                Element::<String>::serialize(tokens.next().unwrap(), tokens)
                        }
                        all @ _ => {
                            //println!("Unimplemented: {all:?}");
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

        entry.authors = if authors.is_empty() {
            None
        } else {
            Some(authors)
        };

        entry.contributors = if contributors.is_empty() {
            None
        } else {
            Some(contributors)
        };

        entry.categories = if categories.is_empty() {
            None
        } else {
            Some(categories)
        };

        entry.links = if links.is_empty() { None } else { Some(links) };

        let data = if attributes.len() > 0 && entry.title.data.is_empty() {
            None
        } else {
            Some(entry)
        };

        Self {
            name: node,
            attributes,
            data,
        }
    }
}
