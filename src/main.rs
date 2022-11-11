// TODO: truncation should be by character instead of by byte
// TODO: use a DB to store posts and classifications
//     - Posts with karma over a specific threshold should get classified as good automatically
//     - Posts that aren't about the lang should be manually verified
// TODO: Have a list of allow-listed channels for youtube links
//     - Youtube links should check if the title is a link, the post is a link, or the body is a
//     link
//     - Probably need to go through the yt api to keep youtube from getting angry
// TODO: Have an allow-list of authors?
//     - This could automatically be updated by keeping count the number of good posts for each
//     author
// TODO: setup a telegram bot for interactions?
// TODO: setup a custom tokenizer. Use a markdown parser when tokenizing
// TODO: setup a naive bayes classifier
// TODO: have a sussy link list?
//     - morioh and tiktok
// TODO: Have an allow list for domains (github.com, github.io, rust-lang.org, crates.io)
//     - This shouldn't be automatically updated because domains serve too many different uses
// TODO: Do IPC over DBUS. Have a daemon (server) and multiple clients (cli and telegram)

// Can snag channel ID for
// ```rust
// TODO: stream the response to keep down memory usage
// fn main() {
//     let body = reqwest::blocking::get("https://youtu.be/rtage0vMbgg")
//         .unwrap()
//         .text()
//         .unwrap();
//     let (_, chunk) = body
//         .split_once(r#"<meta itemprop="channelId" content=""#)
//         .unwrap();
//     let (id, _) = chunk.split_once(r#"">"#).unwrap();
//
//     println!("{id}");
// }
// ```

#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;

mod database;
mod reddit;
mod types;
mod utils;

use std::io;

use tracing_log::LogTracer;
use tracing_subscriber::{filter::LevelFilter, EnvFilter};

const EVENT_LOOP_SLEEP_SEC: u64 = 1 * 60;

fn main() -> anyhow::Result<()> {
    let subscriber = tracing_subscriber::fmt()
        .compact()
        .with_writer(io::stderr)
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .with_env_var("LOG")
                .from_env()?,
        )
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;
    LogTracer::init()?;

    let mut watcher = reddit::Watcher::new();
    let db = database::Database::new()?;
    loop {
        let reddit::Update { fresh, expired } = watcher.update();

        db.insert_posts(expired)?;

        std::thread::sleep(std::time::Duration::from_secs(EVENT_LOOP_SLEEP_SEC));
    }
}
