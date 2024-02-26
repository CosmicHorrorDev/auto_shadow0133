// TODO: Have a list of allow-listed channels for youtube links
//     - Youtube links should check if the title is a link, the post is a link, or the body is a
//     link
//     - Probably need to go through the yt api to keep youtube from getting angry
// TODO: setup a telegram bot for interactions?
// TODO: setup a custom tokenizer. Use a markdown parser when tokenizing

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

use clap::Parser;

#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;

mod cli;
mod commands;
mod config;
mod database;
mod filter;
mod log;
mod reddit;
mod types;
mod utils;

fn main() -> anyhow::Result<()> {
    let _ = dotenv::dotenv();

    log::init()?;

    match cli::Args::parse().command {
        cli::Command::Analyze => commands::analyze::run()?,
        cli::Command::Watch => commands::watch::run()?,
    }

    Ok(())
}
