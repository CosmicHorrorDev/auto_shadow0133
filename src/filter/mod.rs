mod allow_or_block_url;
mod contains_rust_code;
mod known_youtube_channel;
mod reputable_author;

use std::{slice, time::Instant};

use crate::{
    config::Config,
    database::Database,
    types::{Lang, Post},
};

pub struct FilterIter<'ctx> {
    iter: slice::Iter<'static, Filter>,
    context: Context<'ctx>,
}

impl<'ctx> FilterIter<'ctx> {
    pub fn new(post: &'ctx Post, config: &'ctx Config, database: &'ctx Database) -> Self {
        let context = Context {
            post,
            config,
            database,
        };

        Self {
            iter: FILTERS.iter(),
            context,
        }
    }
}

impl Iterator for FilterIter<'_> {
    type Item = (Filter, Option<Status>);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|filter| {
            let start = Instant::now();
            let maybe_status = (filter.1)(self.context);
            tracing::debug!("Filter {} took {:?}", filter.name(), start.elapsed());
            (*filter, maybe_status)
        })
    }
}

#[derive(Clone, Copy)]
pub struct Filter(&'static str, fn(Context) -> Option<Status>);

impl Filter {
    pub fn name(&self) -> &'static str {
        self.0
    }
}

const FILTERS: &[Filter] = &[
    Filter("AllowOrBlockUrl", allow_or_block_url::filter),
    Filter("ReputableAuthor", reputable_author::filter),
    Filter("YoutubeChannel", known_youtube_channel::filter),
    Filter("ContainsRustCode", contains_rust_code::filter),
];

pub fn filter(post: &Post, config: &Config, database: &Database) -> Option<Status> {
    FilterIter::new(post, config, database).find_map(|(_, maybe_status)| maybe_status)
}

#[derive(Clone, Copy)]
pub struct Context<'a> {
    post: &'a Post,
    config: &'a Config,
    database: &'a Database,
}

#[derive(Debug)]
pub enum Status {
    Spam(SpamReason),
    Ham(HamReason),
}

#[derive(Debug)]
pub enum SpamReason {
    BlockedUrl(String),
    BlockedSnippet(String),
    UnknownYoutubeChannel(()),
}

#[derive(Debug)]
pub enum HamReason {
    AllowedUrl(String),
    AllowedSnippet(String),
    DetectedRustCode(contains_rust_code::Heuristic),
    FencedCodeBlock(Lang),
    KnownYoutubeChannel(()),
    ReputableAuthor {
        author: String,
        num_reputable_posts: u32,
    },
}
