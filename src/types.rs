use std::{cmp::Ordering, fmt};

use crate::utils;

use diesel_derive_enum::DbEnum;
use pulldown_cmark::{CodeBlockKind, Event, LinkType, Tag};
use smartstring::alias::String as SmallString;
use time::{format_description::well_known::Rfc3339, OffsetDateTime};

const DEBUG_FIELD_TRUNCATE_LEN: usize = 60;

#[derive(Clone)]
pub struct Post {
    pub id: SmallString,
    pub author: SmallString,
    pub score: f64,
    pub title: String,
    pub created: OffsetDateTime,
    pub body: Option<String>,
    pub link: Option<String>,
    pub category: Option<Category>,
}

#[derive(Debug)]
pub enum Token {
    Code { lang: Option<Lang>, text: String },
    Url { text: Option<String>, url: String },
    Text(String),
}

impl Token {
    fn new<'text>(events: Vec<pulldown_cmark::Event<'text>>) -> Vec<Self> {
        let mut tokens = Vec::new();

        let mut events = events.into_iter();
        while let Some(event) = events.next() {
            match event {
                Event::Start(Tag::Heading(_, Some(t), _)) => tokens.push(Token::Text(t.to_owned())),
                Event::Start(Tag::CodeBlock(code_block_kind)) => {
                    let lang = if let CodeBlockKind::Fenced(lang) = code_block_kind {
                        Lang::new(&lang)
                    } else {
                        None
                    };

                    // Munch text until we see the end codeblock
                    let mut text = String::new();
                    while let Some(event) = events.next() {
                        match event {
                            Event::Text(t) => {
                                if !text.is_empty() {
                                    text.push('\n');
                                }
                                text.push_str(&t);
                            }
                            Event::End(Tag::CodeBlock(_)) => break,
                            // Ignore anything else
                            _ => {}
                        }
                    }

                    tokens.push(Token::Code { lang, text });
                }
                Event::Start(Tag::Link(LinkType::Inline, url, _)) => {
                    let url = url.into_string();

                    let mut text = String::new();
                    while let Some(event) = events.next() {
                        match event {
                            Event::Text(t) => {
                                if !text.is_empty() {
                                    text.push('\n');
                                }
                                text.push_str(&t);
                            }
                            Event::End(Tag::Link(LinkType::Inline, _, _)) => break,
                            // Ignore anything else
                            _ => {}
                        }
                    }
                    tokens.push(Token::Url { text: Some(text), url })
                }
                Event::Text(t) => {
                    // TODO: this still misses plaintext urls that look like
                    // Some content: https://google.com
                    // Should we run a regex over the line, or is it not worth it?
                    let token = if t.starts_with("https://") && t.split_whitespace().count() == 1 {
                        Token::Url { text: None, url: t.into_string() }
                    } else {
                        Token::Text(t.into_string())
                    };

                    tokens.push(token);
                }
                Event::Code(t) => tokens.push(Token::Code { lang: None, text: t.into_string() }),
                // Any significant ends should be consumed with their corresponding start
                Event::End(_) => {}
                // We don't emit tokens for any of these
                Event::Html(_)
                | Event::FootnoteReference(_)
                | Event::SoftBreak
                | Event::HardBreak
                | Event::Rule
                | Event::TaskListMarker(_)
                | Event::Start(
                    Tag::Paragraph
                    | Tag::Heading(_, None, _)
                    | Tag::BlockQuote
                    | Tag::List(_)
                    | Tag::Item
                    | Tag::FootnoteDefinition(_)
                    | Tag::Table(_)
                    | Tag::TableHead
                    | Tag::TableRow
                    | Tag::TableCell
                    | Tag::Emphasis
                    | Tag::Strong
                    | Tag::Strikethrough
                    // Inline links are handled above
                    | Tag::Link(_, _, _)
                    | Tag::Image(_, _, _),
                ) => {}
            }
        }

        tokens
    }
}

#[derive(Debug)]
pub enum Lang {
    Bash,
    C,
    Cpp,
    Go,
    Javascript,
    Python,
    Rust,
    Shell,
}

impl Lang {
    fn new(tag: &str) -> Option<Self> {
        match tag {
            "bash" => Some(Self::Bash),
            "c" => Some(Self::C),
            "cpp" => Some(Self::Cpp),
            "go" => Some(Self::Go),
            "js" => Some(Self::Javascript),
            "python" | "py" => Some(Self::Python),
            "rust" | "rs" => Some(Self::Rust),
            "sh" => Some(Self::Shell),
            _ => None,
        }
    }
}

impl Post {
    pub fn tokens(&self) -> Vec<Token> {
        self.body
            .as_deref()
            .map(|body| {
                let parser = pulldown_cmark::Parser::new(body);
                let events: Vec<_> = parser.into_iter().collect();

                Token::new(events)
            })
            .unwrap_or_default()
    }
}

impl PartialEq for Post {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Post {}

impl PartialOrd for Post {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.id.partial_cmp(&other.id)
    }
}

impl Ord for Post {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl fmt::Debug for Post {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            id,
            author,
            score,
            title,
            created,
            body,
            link,
            category,
        } = &self;

        let mut debug_struct = f.debug_struct("Post");
        debug_struct.field("id", id);
        debug_struct.field("author", author);
        debug_struct.field("score", score);

        let truncated_title = utils::truncate_str(title, DEBUG_FIELD_TRUNCATE_LEN);
        debug_struct.field("title", &truncated_title);

        debug_struct.field("created", &created.format(&Rfc3339).unwrap());

        let truncated_body = body
            .as_deref()
            .map(|b| utils::truncate_str(b, DEBUG_FIELD_TRUNCATE_LEN));
        debug_struct.field("body", &truncated_body);

        debug_struct.field("link", link);
        debug_struct.field("category", category);

        debug_struct.finish()
    }
}

#[derive(DbEnum, Clone, Copy, Debug)]
pub enum Category {
    Lang,
    Game,
    Other,
}
