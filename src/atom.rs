use std::iter::Peekable;

use greyxml::{lex, tokenize, Tokens};

use crate::elements::{self, Element};

type AtomResult<T> = Result<T, Box<dyn std::error::Error>>;

// https://datatracker.ietf.org/doc/html/rfc4287#section-4.1.1
#[derive(Debug, Default)]
pub struct Feed {
    pub specs: Option<Vec<(String, String)>>,
    pub contents: Inner,
}

impl Feed {
    pub fn serialize(input: &str) -> AtomResult<Feed> {
        let lexed = lex(input.into())?;
        let tokens = tokenize(&mut lexed.into_iter())?;

        let mut tokens_iter = tokens.into_iter().peekable();

        let Some(feed) = tokens_iter.find(|t| t == &Tokens::OpenNode("feed".into())) else {
            //FIXME: Handle error properly
            panic!("Could not find rss node");
        };

        let Some(token) = tokens_iter.next() else {
            panic!("No more tokens?");
        };

        let mut specs = vec![];

        while let Some(token) = tokens_iter.peek() {
            let token = token.clone();
            match token {
                Tokens::ParameterName(name) => {
                    tokens_iter.next();
                    if let Some(Tokens::ParameterValue(value)) = tokens_iter.next() {
                        specs.push((name.clone(), value));
                    }
                }
                _ => {
                    break;
                }
            }
        }

        let inner = Inner::serialize(&mut tokens_iter);

        Ok(Feed {
            specs: if specs.is_empty() { None } else { Some(specs) },
            // FIXME: Don't use `unwrap()` here
            contents: inner,
        })
    }
}

#[derive(Debug, Default)]
pub struct Inner {
    // MUST contain one or more unless all entries have an author element
    // https://datatracker.ietf.org/doc/html/rfc4287#section-4.2.1
    pub authors: Option<Vec<Element<Person>>>,
    // MAY contain any number of
    pub categories: Option<Vec<Element<Category>>>,
    // MAY contain any number of
    pub contributors: Option<Vec<Element<Person>>>,
    // MUST NOT contain more than one
    // https://datatracker.ietf.org/doc/html/rfc4287#section-4.2.4
    pub generator: Option<Element<String>>,
    // MUST NOT contain more than one
    pub icon: Option<Element<String>>,
    // MUST contain exactly one
    pub id: Element<String>,
    // "SHOULD" contain one with a rel of "self"
    // MUST NOT contain more than one link with rel "anternate" that has the
    // same combination of type and hreflang attribute values.
    // MAY contain additional links besides those above
    pub links: Option<Vec<Element<Link>>>,
    // MOST NOT contain more than one
    pub logo: Option<Element<String>>,
    // MUST NOT contain more than one
    // https://datatracker.ietf.org/doc/html/rfc4287#section-4.2.10
    pub rights: Option<Element<String>>,
    // MUST NOT contain more than one
    pub subtitle: Option<Element<String>>,
    // MUST contain exactly one
    pub title: Element<String>,
    // MUST contain exactly one
    pub updated: Element<String>,
    pub entries: Vec<Element<Option<Entry>>>,
}
impl Inner {
    pub fn serialize<I>(tokens: &mut Peekable<I>) -> Self
    where
        I: std::iter::Iterator<Item = Tokens>,
    {
        let mut attributes = vec![];

        let mut inner = Inner::default();

        let mut looking_for_attributes = true;

        let mut authors = vec![];
        let mut categories = vec![];
        let mut contributors = vec![];
        let mut links = vec![];
        let mut entries = vec![];

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
                        "author" => authors
                            .push(Element::<Person>::serialize(tokens.next().unwrap(), tokens)),
                        "category" => categories.push(Element::<Category>::serialize(
                            tokens.next().unwrap(),
                            tokens,
                        )),
                        "contributor" => contributors
                            .push(Element::<Person>::serialize(tokens.next().unwrap(), tokens)),
                        "generator" => {
                            inner.generator =
                                Some(Element::<String>::serialize(tokens.next().unwrap(), tokens))
                        }
                        "icon" => {
                            inner.icon =
                                Some(Element::<String>::serialize(tokens.next().unwrap(), tokens))
                        }
                        "id" => {
                            inner.id = Element::<String>::serialize(tokens.next().unwrap(), tokens)
                        }
                        "link" => {
                            links.push(Element::<Link>::serialize(tokens.next().unwrap(), tokens))
                        }
                        "logo" => {
                            inner.logo =
                                Some(Element::<String>::serialize(tokens.next().unwrap(), tokens))
                        }
                        "rights" => {
                            inner.rights =
                                Some(Element::<String>::serialize(tokens.next().unwrap(), tokens))
                        }
                        "subtitle" => {
                            inner.subtitle =
                                Some(Element::<String>::serialize(tokens.next().unwrap(), tokens))
                        }
                        "title" => {
                            inner.title =
                                Element::<String>::serialize(tokens.next().unwrap(), tokens)
                        }
                        "updated" => {
                            inner.updated =
                                Element::<String>::serialize(tokens.next().unwrap(), tokens)
                        }
                        "entry" => entries.push(Element::<Option<Entry>>::serialize(
                            tokens.next().unwrap(),
                            tokens,
                        )),
                        all @ _ => {
                            println!("Unimplemented: {all:?}");
                        }
                    }
                }
                Tokens::CloseNode(close) => {
                    if close == "feed" {
                        break;
                    }
                }
                _ => {
                    looking_for_attributes = false;
                }
            }
            tokens.next();
        }

        inner.authors = if authors.is_empty() {
            None
        } else {
            Some(authors)
        };
        inner.categories = if categories.is_empty() {
            None
        } else {
            Some(categories)
        };
        inner.contributors = if contributors.is_empty() {
            None
        } else {
            Some(contributors)
        };
        inner.links = if links.is_empty() { None } else { Some(links) };
        inner.entries = entries;

        inner
    }
}

#[derive(Debug, Default)]
// https://datatracker.ietf.org/doc/html/rfc4287#section-4.1.2
pub struct Entry {
    pub authors: Option<Vec<Element<Person>>>,
    pub categories: Option<Vec<Element<Category>>>,
    pub content: Element<String>,
    pub contributors: Option<Vec<Element<Person>>>,
    pub id: Element<String>,
    // atom:entry elements that contain no child atom:content element
    // MUST contain at least one atom:link element with a rel attribute
    // value of "alternate".
    pub links: Option<Vec<Element<Link>>>,
    pub published: Option<Element<String>>,
    pub rights: Option<Element<String>>,
    pub summary: Option<Element<String>>,
    pub title: Element<String>,
    pub updated: Element<String>,
}

#[derive(Debug, Default)]
// https://datatracker.ietf.org/doc/html/rfc4287#section-4.2.2
// Contains only attributes it seems?
pub struct Category;

#[derive(Debug, Default)]
// https://datatracker.ietf.org/doc/html/rfc4287#section-3.2
pub struct Person {
    pub name: Element<String>,
    pub uri: Option<Element<String>>,
    pub email: Option<Element<String>>,
}

#[derive(Debug, Default)]
// https://datatracker.ietf.org/doc/html/rfc4287#section-4.2.7
pub struct Link(pub Option<String>);
