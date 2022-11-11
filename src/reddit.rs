// TODO: add in tests

use std::collections::{BTreeSet, VecDeque};

use crate::{
    types::{Post, PostKind},
    utils,
};

use roux::{submission::SubmissionData, util::RouxError, Subreddit};
use smartstring::alias::String as SmallString;
use time::OffsetDateTime;
use url::Url;

#[derive(Debug, Default)]
pub struct Update {
    pub fresh: Vec<Post>,
    pub expired: Vec<Post>,
}

impl Update {
    pub fn is_empty(&self) -> bool {
        self.fresh.is_empty() && self.expired.is_empty()
    }
}

trait PostSource {
    fn posts(&self) -> Result<BTreeSet<Post>, RouxError>;
}

struct RustSubreddit {
    inner: Subreddit,
}

impl RustSubreddit {
    fn new() -> Self {
        Self {
            inner: Subreddit::new("rust"),
        }
    }
}

impl PostSource for RustSubreddit {
    fn posts(&self) -> Result<BTreeSet<Post>, RouxError> {
        self.inner
            .latest(NUM_LATEST_POSTS, None)
            .map(|submissions| {
                submissions
                    .data
                    .children
                    .into_iter()
                    .map(|container| Post::from(container.data))
                    .collect()
            })
    }
}

const NUM_LATEST_POSTS: u32 = 20;
const ID_BUFFER: usize = NUM_LATEST_POSTS as usize + 100;

pub struct Watcher {
    source: Box<dyn PostSource>,
    live: BTreeSet<Post>,
    // Posts being removed can cause already fresh/expired posts to be re-emitted. Keep track of a
    // longer queue to keep from emitting more than once (unless a ridiculous amount of posts are
    // removed)
    fresh_debounce: VecDeque<SmallString>,
    expired_debounce: VecDeque<SmallString>,
}

impl Watcher {
    pub fn new() -> Self {
        let source = Box::new(RustSubreddit::new());
        let live = BTreeSet::new();
        let fresh_debounce = VecDeque::with_capacity(ID_BUFFER);
        let expired_debounce = VecDeque::with_capacity(ID_BUFFER);

        Self {
            source,
            live,
            fresh_debounce,
            expired_debounce,
        }
    }

    pub fn update(&mut self) -> Update {
        fn update_post_listing(
            left: &BTreeSet<Post>,
            right: &BTreeSet<Post>,
            debounce_ids: &mut VecDeque<SmallString>,
        ) -> Vec<Post> {
            let diff: Vec<_> = left
                .difference(right)
                .filter(|post| !debounce_ids.contains(&post.id))
                .cloned()
                .collect();

            for post in &diff {
                // There shouldn't ever be more ids than `ID_BUFFER`, but loop just to be safe
                while debounce_ids.len() >= ID_BUFFER {
                    debounce_ids.pop_back();
                }

                debounce_ids.push_front(post.id.clone());
            }

            diff
        }

        match self.source.posts() {
            Ok(latest) => {
                // Find posts that are newly included in `.lastest()`
                let fresh = update_post_listing(&latest, &self.live, &mut self.fresh_debounce);
                // Find posts that were in `.latest()`, but aren't now
                let expired = update_post_listing(&self.live, &latest, &mut self.expired_debounce);

                // Update stored data
                self.live = latest;

                let update = Update { fresh, expired };
                if !update.is_empty() {
                    // HACK: hopefully tracing gets support for pretty-debug sometime
                    tracing::debug!(update = %format!("{update:#?}"), "Emitting update");

                    let log_format_titles = |posts: &[Post]| {
                        let titles: Vec<_> = posts
                            .iter()
                            .map(|post| utils::truncate_str(&post.title, 80))
                            .collect();
                        format!("{titles:#?}")
                    };
                    tracing::info!(
                        fresh = %log_format_titles(&update.fresh),
                        expired = %log_format_titles(&update.expired),
                        "Update summary"
                    );
                }
                update
            }
            Err(e) => {
                tracing::warn!(%e, "You burnt the roux ;-;");
                Update::default()
            }
        }
    }
}

impl From<SubmissionData> for Post {
    fn from(
        SubmissionData {
            id,
            author,
            score,
            title,
            selftext,
            is_self,
            created_utc,
            url,
            ..
        }: SubmissionData,
    ) -> Self {
        let created = OffsetDateTime::from_unix_timestamp(created_utc as i64).unwrap();
        let selftext = selftext.trim();
        let title = title.trim();

        let maybe_url = if is_self {
            // If the title is a link and there is no body, or the body is just a link then
            // consider it a link post
            if selftext.is_empty() && Url::parse(title).is_ok() {
                Some(title.to_owned())
            } else if Url::parse(selftext).is_ok() {
                Some(selftext.to_owned())
            } else {
                None
            }
        } else {
            url
        };

        let (kind, body) = match maybe_url {
            Some(url) => (PostKind::Link, url),
            None => (PostKind::Text, selftext.to_owned()),
        };

        Self {
            id: SmallString::from(id),
            author: SmallString::from(author),
            score,
            title: title.to_owned(),
            created,
            kind,
            body,
        }
    }
}
