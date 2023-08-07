#![feature(let_chains)]

use std::env;
use std::io::{self, Read, Stdout};
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use std::{error::Error, fs::File};

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers, ModifierKeyCode};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use greyxml::{lex, tokenize};
use ratatui::prelude::*;
use ratatui::widgets::*;
use reqwest::blocking::get;

mod atom;
mod elements;
mod rss;

#[derive(Clone)]
struct StatefulList<T> {
    items: Vec<T>,
    state: ListState,
    /// Is this the currently active list
    active: bool,
    // TODO: May need value for currently highlighted
}

impl<T> StatefulList<T> {
    fn with_items(items: Vec<T>, is_active: bool) -> StatefulList<T> {
        let mut state = ListState::default();
        if items.len() > 0 {
            state.select(Some(0));
        }
        StatefulList {
            items,
            state,
            active: is_active,
        }
    }

    fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn unselect(&mut self) { self.state.select(None); }
}

impl<String: std::fmt::Display> StatefulList<String> {
    fn to_list(&self) -> List {
        let list_items: Vec<ListItem> = self
            .items
            .iter()
            .enumerate()
            .map(|(index, item)| {
                if let Some(i) = self.state.selected() && i == index {
                    if self.active {
                        ListItem::new(format!("**{item}**")).italic()
                    } else {
                        ListItem::new(format!("**{item}**"))
                    }
                } else {
                    ListItem::new(item.to_string())
                }
            })
            .collect();
        List::new(list_items)
    }
}

impl StatefulList<(String, usize)> {
    fn to_list_tuple(&self) -> List {
        let list_items: Vec<ListItem> = self
            .items
            .iter()
            .enumerate()
            .map(|(index, item)| {
                if let Some(i) = self.state.selected() && i == index {
                    if self.active {
                        ListItem::new(format!("**{}**", item.0)).italic()
                    } else {
                        ListItem::new(format!("**{}**", item.0))
                    }
                } else {
                    ListItem::new(item.0.to_string())
                }
            })
            .collect();
        List::new(list_items)
    }
}

enum FeedType {
    Rss(rss::Feed),
    Atom(atom::Feed),
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut input = String::new();
    let mut args = env::args();
    args.next();

    let mut feeds = vec![];

    for _ in 0..args.len() {
        let Some(path_string) = args.next() else {
            panic!("No path given");
        };
        if path_string.starts_with("http://") || path_string.starts_with("https://") {
            // FIXME: Handle Errors
            input = get_web_feed(&path_string)?;
        } else {
            let path = PathBuf::from(path_string.clone());
            let mut file = File::open(path)?;
            file.read_to_string(&mut input)?;
        }
        if path_string.ends_with(".atom") {
            feeds.push(FeedType::Atom(atom::Feed::serialize(&input)?));
        } else {
            feeds.push(FeedType::Rss(rss::Feed::serialize(&input)?));
        }
    }

    /*
    let lexed = lex(input.into())?;
    let tokens = tokenize(&mut lexed.into_iter())?;

    dbg!(tokens);
    */

    //let feed = rss::Feed::serialize(&input)?;
    //dbg!(feed);

    let mut terminal = setup_terminal()?;
    match run(&mut terminal, feeds) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Error Occured: {e}");
        }
    }

    restore_terminal(&mut terminal)?;

    Ok(())
}

fn get_web_feed(source: &str) -> Result<String, Box<dyn Error>> {
    let body = get(source)?.text()?;
    Ok(body)
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>, Box<dyn Error>> {
    let mut stdout = io::stdout();
    enable_raw_mode()?;
    execute!(stdout, EnterAlternateScreen)?;
    Ok(Terminal::new(CrosstermBackend::new(stdout))?)
}

fn restore_terminal(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
) -> Result<(), Box<dyn Error>> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(terminal.show_cursor()?)
}

fn run(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    feeds: Vec<FeedType>,
) -> Result<(), Box<dyn Error>> {
    let mut feeds_index: StatefulList<String> =
        StatefulList::with_items(vec!["Kisserss".into(), "DaisyUniverse".into()], true);

    let mut feeds_index = StatefulList::with_items(
        feeds
            .iter()
            .enumerate()
            .map(|(index, feed)| match feed {
                FeedType::Rss(rss) => (rss.channel.data.title.data.clone(), index),
                FeedType::Atom(atom) => (atom.contents.title.data.clone(), index),
            })
            .collect(),
        true,
    );

    let Some(active_feed_index) = feeds_index.state.selected() else {
        panic!("No feed was active by default");
    };

    let mut feed_items = StatefulList::with_items(
        match feeds[active_feed_index] {
            FeedType::Rss(ref rss) => rss
                .channel
                .data
                .items
                .iter()
                .enumerate()
                .map(|(index, item)| {
                    if let Some(ref title) = item.data.title {
                        (title.data.clone(), index)
                    } else {
                        if let Some(ref date) = item.data.pub_date {
                            (date.data.clone(), index)
                        } else {
                            (String::new(), 0)
                        }
                    }
                })
                .filter(|(string, _)| !string.is_empty())
                .collect(),
            FeedType::Atom(ref atom) => atom
                .contents
                .entries
                .iter()
                .enumerate()
                .map(|(index, item)| {
                    if let Some(ref entry) = item.data {
                        (entry.title.data.clone(), index)
                    } else {
                        (String::new(), index)
                    }
                })
                .filter(|(string, _)| !string.is_empty())
                .collect(),
        },
        false,
    );
    let global_block = Block::new().borders(Borders::ALL).title("Kisserss");
    let inner_block = Block::new().borders(Borders::TOP);
    let feeds_block = Block::new().borders(Borders::RIGHT);
    let content_block = Block::new().borders(Borders::TOP).title("Content");

    let mut active_window = 0;
    let mut active_feed_changed = false;

    Ok(loop {
        terminal.draw(|frame| {
            let Some(active_feed_index) = feeds_index.state.selected() else {
                panic!("No feed was active");
            };
            // FIXME: THIS IS NOT IDEAL Move into App State
            if active_window == 0 {
                feeds_index.active = true;
                feed_items.active = false;
                if active_feed_changed {
                    feed_items = StatefulList::with_items(
                        match feeds[active_feed_index] {
                            FeedType::Rss(ref rss) => rss
                                .channel
                                .data
                                .items
                                .iter()
                                .enumerate()
                                .map(|(index, item)| {
                                    if let Some(ref title) = item.data.title {
                                        (title.data.clone(), index)
                                    } else {
                                        if let Some(ref date) = item.data.pub_date {
                                            (date.data.clone(), index)
                                        } else {
                                            (String::new(), 0)
                                        }
                                    }
                                })
                                .filter(|(string, _)| !string.is_empty())
                                .collect(),
                            FeedType::Atom(ref atom) => atom
                                .contents
                                .entries
                                .iter()
                                .enumerate()
                                .map(|(index, item)| {
                                    if let Some(ref entry) = item.data {
                                        (entry.title.data.clone(), index)
                                    } else {
                                        (String::new(), index)
                                    }
                                })
                                .filter(|(string, _)| !string.is_empty())
                                .collect(),
                        },
                        false,
                    );
                    active_feed_changed = false;
                }
            } else if active_window == 1 {
                feeds_index.active = false;
                feed_items.active = true;
            } else {
                feeds_index.active = false;
                feed_items.active = false;
            }
            let outer_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Max(1), Constraint::Min(1)].as_ref())
                .split(global_block.inner(frame.size()));
            let inner_layout = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
                .margin(0)
                .split(inner_block.inner(outer_layout[1]));
            let content_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
                .split(inner_layout[1]);

            let feeds_clone = feeds_index.clone();
            let feed_items_clone = feed_items.clone();

            let feeds_list = feeds_clone.to_list_tuple();
            let items_list = feed_items_clone.to_list_tuple();

            let content = if let Some(selected) = feed_items.state.selected() {
                let feed = &feeds[active_feed_index];
                match feed {
                    FeedType::Rss(rss) => {
                        if let Some(ref desc) = rss.channel.data.items[feed_items.items[selected].1]
                            .data
                            .description
                        {
                            Paragraph::new(desc.data.clone())
                        } else {
                            Paragraph::new(String::new())
                        }
                    }
                    FeedType::Atom(atom) => {
                        if let Some(ref entry) =
                            atom.contents.entries[feed_items.items[selected].1].data
                        {
                            Paragraph::new(entry.content.data.clone())
                        } else {
                            Paragraph::new(String::new())
                        }
                    }
                }
            } else {
                Paragraph::new(String::new())
            };

            frame.render_widget(global_block.clone(), frame.size());
            frame.render_widget(inner_block.clone(), outer_layout[1]);
            frame.render_widget(Paragraph::new("F1: Add Feed"), outer_layout[0]);
            frame.render_stateful_widget(
                feeds_list.block(feeds_block.clone()),
                inner_layout[0],
                &mut feeds_index.state.clone(),
            );
            frame.render_stateful_widget(items_list, content_layout[0], &mut feed_items.state);
            frame.render_widget(
                content
                    .block(content_block.clone())
                    .wrap(Wrap { trim: false }),
                content_layout[1],
            );
        })?;
        if event::poll(Duration::from_millis(250))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            break;
                        }
                        KeyCode::Char('s') | KeyCode::Down => {
                            if active_window == 0 {
                                feeds_index.next();
                                active_feed_changed = true;
                            } else if active_window == 1 {
                                feed_items.next();
                            }
                        }
                        KeyCode::Char('w') | KeyCode::Up => {
                            if active_window == 0 {
                                feeds_index.previous();
                                active_feed_changed = true;
                            } else if active_window == 1 {
                                feed_items.previous();
                            }
                        }
                        KeyCode::Tab => {
                            if active_window == 2 {
                                active_window = 0;
                                continue;
                            }
                            active_window += 1;
                        }
                        _ => {}
                    }
                }
            }
        }
    })
}
