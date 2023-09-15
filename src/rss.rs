use crate::elements::Element;
use greyxml::{lex, tokenize, Tokens};

type RssResult<T> = Result<T, Box<dyn std::error::Error>>;

#[derive(Debug)]
pub struct Feed {
    pub version: f32,
    pub specs: Option<Vec<(String, String)>>,
    pub channel: Element<Channel>,
}

impl Feed {
    pub fn serialize(input: &str) -> RssResult<Feed> {
        let lexed = lex(input.into())?;
        let tokens = tokenize(&mut lexed.into_iter())?;

        let mut tokens_iter = tokens.into_iter().peekable();

        let Some(rss) = tokens_iter.find(|t| t == &Tokens::OpenNode("rss".into())) else {
            //FIXME: Handle error properly
            panic!("Could not find rss node");
        };

        if let None = tokens_iter.peek() {
            panic!("No more tokens?");
        }

        let mut version = 0.0;
        let mut specs = vec![];

        while let Some(token) = tokens_iter.peek() {
            let token = token.clone();
            match token {
                Tokens::ParameterName(name) => {
                    if name == "version" {
                        tokens_iter.next();
                        if let Some(Tokens::ParameterValue(v)) = tokens_iter.next() {
                            version = v.parse().unwrap();
                        } else {
                            version = 0.0;
                        }
                        continue;
                    }
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

        Ok(Feed {
            version,
            specs: if specs.is_empty() { None } else { Some(specs) },
            // FIXME: Don't use `unwrap()` here
            channel: Element::<Channel>::serialize(tokens_iter.next().unwrap(), &mut tokens_iter),
        })
    }
}

// https://www.rssboard.org/rss-specification
#[derive(Default, Debug)]
pub struct Channel {
    pub title: Element<String>,
    pub link: Element<String>,
    pub description: Element<String>,
    pub items: Vec<Element<Item>>,
    pub language: Option<Element<String>>,
    pub copyright: Option<Element<String>>,
    pub managing_editor: Option<Element<String>>,
    pub web_master: Option<Element<String>>,
    // TODO: Maybe make this a chrono time thing
    pub pub_date: Option<Element<String>>,
    pub last_build_date: Option<Element<String>>,
    pub category: Option<Element<String>>,
    pub generator: Option<Element<String>>,
    pub docs: Option<Element<String>>,
    pub cloud: Option<Element<String>>,
    pub ttl: Option<Element<u32>>,
    pub image: Option<Element<Image>>,
    // FIXME: Idk what the format of this is yet
    pub rating: Option<Element<String>>,
    pub text_input: Option<Element<TextInput>>,
    pub skip_hours: Option<Element<SkipHours>>,
    pub skip_days: Option<Element<SkipDays>>,
}

// https://www.rssboard.org/rss-specification#hrelementsOfLtitemgt
#[derive(Default, Debug)]
pub struct Item {
    pub title: Option<Element<String>>,
    pub link: Option<Element<String>>,
    pub description: Option<Element<String>>,
    pub author: Option<Element<String>>,
    pub category: Option<Element<String>>,
    pub enclosure: Option<Element<String>>,
    pub guid: Option<Element<String>>,
    pub pub_date: Option<Element<String>>,
    pub source: Option<Element<String>>,
    // part of https://web.resource.org/rss/1.0/modules/content/
    pub content_encoded: Option<Element<EncodedContent>>,
    // part of https://www.rssboard.org/media-rss
    pub media: Option<Element<Media>>,
}

#[derive(Default, Debug)]
pub struct EncodedContent(pub Vec<Element<String>>, pub(crate) String);

#[derive(Default, Debug)]
pub struct Image {
    pub url: Element<String>,
    pub title: Element<String>,
    pub link: Element<String>,
    pub width: Option<Element<u32>>,
    pub height: Option<Element<u32>>,
    pub description: Option<Element<String>>,
}

#[derive(Default, Debug)]
pub struct TextInput {
    pub title: Element<String>,
    pub description: Element<String>,
    pub name: Element<String>,
    pub link: Element<String>,
}

#[derive(Default, Debug)]
pub struct SkipHours {
    // A number between 0 and 23
    pub hours: [u8; 24],
}

#[derive(Default, Debug)]
pub enum Days {
    #[default]
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}

#[derive(Default, Debug)]
pub struct SkipDays {
    pub days: [Days; 7],
}

#[derive(Default, Debug)]
pub struct Media {}