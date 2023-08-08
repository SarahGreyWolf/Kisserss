#![feature(let_chains)]

use std::env;
use std::io::{self, Read, Stdout, Write};
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
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

#[derive(Debug)]
enum FeedType {
    Rss(rss::Feed),
    Atom(atom::Feed),
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut input = String::new();
    let mut args = env::args();
    args.next();

    let mut feeds = vec![];

    // TODO: Multithreaded loading?
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

    //let feed = &feeds[0];
    //dbg!(feed);

    let mut terminal = setup_terminal()?;
    let mut app = App::new(feeds, &mut terminal);
    app.run()?;

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

struct App<'a> {
    feeds: Vec<FeedType>,
    feeds_list: StatefulList<(String, usize)>,
    active_feed: usize,
    feed_items: StatefulList<(String, usize)>,
    active_item: usize,
    active_window: usize,
    active_feed_changed: bool,
    terminal: &'a mut Terminal<CrosstermBackend<Stdout>>,
}

impl<'a> App<'a> {
    pub fn new(
        feeds: Vec<FeedType>,
        terminal: &'a mut Terminal<CrosstermBackend<Stdout>>,
    ) -> App<'a> {
        let feeds_list = StatefulList::with_items(
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
        let Some(active_feed_index) = feeds_list.state.selected() else {
            panic!("No feed was active by default");
        };
        let feed_items = StatefulList::with_items(
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

        Self {
            feeds,
            feeds_list,
            active_feed: active_feed_index,
            feed_items,
            active_item: 0,
            active_window: 0,
            active_feed_changed: false,
            terminal,
        }
    }

    fn run(&mut self) -> Result<(), Box<dyn Error>> {
        loop {
            let Some(active_feed_index) = self.feeds_list.state.selected() else {
                panic!("No feed was active");
            };
            self.active_feed = active_feed_index;
            if self.active_window == 0 {
                self.feeds_list.active = true;
                self.feed_items.active = false;
                if self.active_feed_changed {
                    self.feed_items = StatefulList::with_items(
                        match self.feeds[self.active_feed] {
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
                    self.active_feed_changed = false;
                }
            } else if self.active_window == 1 {
                self.feeds_list.active = false;
                self.feed_items.active = true;
            } else {
                self.feeds_list.active = false;
                self.feed_items.active = false;
            }
            if self.events()? {
                break;
            }
            self.render()?;
        }
        Ok(())
    }

    fn events(&mut self) -> Result<bool, Box<dyn Error>> {
        if event::poll(Duration::from_millis(250))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') => return Ok(true),
                        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            return Ok(true);
                        }
                        KeyCode::Char('s') | KeyCode::Down => {
                            if self.active_window == 0 {
                                self.feeds_list.next();
                                self.active_feed_changed = true;
                            } else if self.active_window == 1 {
                                self.feed_items.next();
                            }
                        }
                        KeyCode::Char('w') | KeyCode::Up => {
                            if self.active_window == 0 {
                                self.feeds_list.previous();
                                self.active_feed_changed = true;
                            } else if self.active_window == 1 {
                                self.feed_items.previous();
                            }
                        }
                        KeyCode::Tab => {
                            if self.active_window == 2 {
                                self.active_window = 0;
                                return Ok(false);
                            }
                            self.active_window += 1;
                        }
                        _ => {}
                    }
                }
            }
        }
        Ok(false)
    }

    fn render(&mut self) -> Result<(), Box<dyn Error>> {
        self.terminal.draw(|f| {
            let global_block = Block::new().borders(Borders::ALL).title("Kisserss");
            let inner_block = Block::new().borders(Borders::TOP);
            let feeds_block = Block::new().borders(Borders::RIGHT);
            let content_block = Block::new().borders(Borders::TOP).title("Content");
            let outer_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Max(1), Constraint::Min(1)].as_ref())
                .split(global_block.inner(f.size()));
            let inner_layout = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
                .margin(0)
                .split(inner_block.inner(outer_layout[1]));
            let content_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
                .split(inner_layout[1]);

            let feeds_clone = self.feeds_list.clone();
            let feed_items_clone = self.feed_items.clone();

            let feeds_list = feeds_clone.to_list_tuple();
            let items_list = feed_items_clone.to_list_tuple();

            let content = if let Some(selected) = self.feed_items.state.selected() {
                let feed = &self.feeds[self.active_feed];
                match feed {
                    FeedType::Rss(rss) => {
                        if let Some(ref desc) = rss.channel.data.items
                            [self.feed_items.items[selected].1]
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
                            atom.contents.entries[self.feed_items.items[selected].1].data
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

            f.render_widget(global_block.clone(), f.size());
            f.render_widget(inner_block.clone(), outer_layout[1]);
            f.render_widget(Paragraph::new("F1: Add Feed"), outer_layout[0]);
            f.render_stateful_widget(
                feeds_list.block(feeds_block.clone()),
                inner_layout[0],
                &mut self.feeds_list.state.clone(),
            );
            f.render_stateful_widget(items_list, content_layout[0], &mut self.feed_items.state);
            f.render_widget(
                content
                    .block(content_block.clone())
                    .wrap(Wrap { trim: false }),
                content_layout[1],
            );
        })?;
        Ok(())
    }
}
