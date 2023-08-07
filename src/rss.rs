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

        let Some(token) = tokens_iter.next() else {
            panic!("No more tokens?");
        };

        let version: f32 = if token == Tokens::ParameterName("version".into()) {
            if let Some(Tokens::ParameterValue(v)) = tokens_iter.next() {
                v.parse().unwrap()
            } else {
                0.0
            }
        } else {
            0.0
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
    pub(crate) title: Element<String>,
    pub(crate) link: Element<String>,
    pub(crate) description: Element<String>,
    pub(crate) items: Vec<Element<Item>>,
    pub(crate) language: Option<Element<String>>,
    pub(crate) copyright: Option<Element<String>>,
    pub(crate) managing_editor: Option<Element<String>>,
    pub(crate) web_master: Option<Element<String>>,
    // TODO: Maybe make this a chrono time thing
    pub(crate) pub_date: Option<Element<String>>,
    pub(crate) last_build_date: Option<Element<String>>,
    pub(crate) category: Option<Element<String>>,
    pub(crate) generator: Option<Element<String>>,
    pub(crate) docs: Option<Element<String>>,
    pub(crate) cloud: Option<Element<String>>,
    pub(crate) ttl: Option<Element<u32>>,
    pub(crate) image: Option<Element<Image>>,
    // FIXME: Idk what the format of this is yet
    pub(crate) rating: Option<Element<String>>,
    pub(crate) text_input: Option<Element<TextInput>>,
    pub(crate) skip_hours: Option<Element<SkipHours>>,
    pub(crate) skip_days: Option<Element<SkipDays>>,
}

// https://www.rssboard.org/rss-specification#hrelementsOfLtitemgt
#[derive(Default, Debug)]
pub struct Item {
    pub(crate) title: Option<Element<String>>,
    pub(crate) link: Option<Element<String>>,
    pub(crate) description: Option<Element<String>>,
    pub(crate) author: Option<Element<String>>,
    pub(crate) category: Option<Element<String>>,
    pub(crate) enclosure: Option<Element<String>>,
    pub(crate) guid: Option<Element<String>>,
    pub(crate) pub_date: Option<Element<String>>,
    pub(crate) source: Option<Element<String>>,
    // part of https://web.resource.org/rss/1.0/modules/content/
    pub(crate) content_encoded: Option<Element<EncodedContent>>,
    // part of https://www.rssboard.org/media-rss
    pub(crate) media: Option<Element<Media>>,
}

#[derive(Default, Debug)]
pub struct EncodedContent(pub(crate) Vec<Element<String>>, pub(crate) String);

#[derive(Default, Debug)]
pub struct Image {
    pub(crate) url: Element<String>,
    pub(crate) title: Element<String>,
    pub(crate) link: Element<String>,
    pub(crate) width: Option<Element<u32>>,
    pub(crate) height: Option<Element<u32>>,
    pub(crate) description: Option<Element<String>>,
}

#[derive(Default, Debug)]
pub struct TextInput {
    pub(crate) title: Element<String>,
    pub(crate) description: Element<String>,
    pub(crate) name: Element<String>,
    pub(crate) link: Element<String>,
}

#[derive(Default, Debug)]
pub struct SkipHours {
    // A number between 0 and 23
    pub(crate) hours: [u8; 24],
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
    pub(crate) days: [Days; 7],
}

#[derive(Default, Debug)]
pub struct Media {}
