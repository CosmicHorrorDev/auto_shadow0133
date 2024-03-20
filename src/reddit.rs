// TODO: add in tests

use std::collections::{BTreeSet, VecDeque};

use crate::{types::Post, utils};

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
    fn posts(&mut self) -> Result<BTreeSet<Post>, RouxError>;
}

struct RustSubreddit {
    inner: Subreddit,
}

impl RustSubreddit {
    fn new() -> Self {
        let secrets = crate::config::expect_secrets();
        let user_agent = concat!(
            "AutoShadow0133:",
            env!("CARGO_PKG_VERSION"),
            " from https://github.com/CosmicHorrorDev/auto_shadow0133"
        );
        let inner = roux::Reddit::new(
            user_agent,
            &secrets.reddit.client_id,
            &secrets.reddit.client_secret,
        )
        .username(&secrets.reddit.username)
        .password(&secrets.reddit.password)
        .subreddit("rust")
        .unwrap();
        Self { inner }
    }
}

impl PostSource for RustSubreddit {
    fn posts(&mut self) -> Result<BTreeSet<Post>, RouxError> {
        fn try_fetch(sub: &Subreddit) -> Result<BTreeSet<Post>, RouxError> {
            sub.latest(NUM_LATEST_POSTS, None).map(|submissions| {
                submissions
                    .data
                    .children
                    .into_iter()
                    .map(|container| Post::from(container.data))
                    .collect()
            })
        }

        try_fetch(&self.inner).or_else(|_| {
            // Refresh our auth and try again on failure since it seems like auth can start failing
            // over time
            tracing::info!("Attempting to refresh auth");
            *self = Self::new();
            try_fetch(&self.inner)
        })
    }
}

const NUM_LATEST_POSTS: u32 = 20;
const ID_BUFFER: usize = NUM_LATEST_POSTS as usize + 300;

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
            Err(error) => {
                tracing::warn!(%error, "You burnt the roux ;-;");
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
            created_utc,
            url: mut maybe_url,
            ..
        }: SubmissionData,
    ) -> Self {
        let created = OffsetDateTime::from_unix_timestamp(created_utc as i64).unwrap();
        let selftext = selftext.trim();
        let selftext = (!selftext.is_empty()).then_some(selftext);
        let title = title.trim();

        if maybe_url.is_none() {
            // Some people mess up link posting, so if the title is a link and there is no body, or
            // there is a title, but the body is just a link then consider it a link post
            let check_as_url = selftext.unwrap_or(title);
            if Url::parse(check_as_url).is_ok() {
                maybe_url = Some(check_as_url.to_owned());
            }
        }

        maybe_url = maybe_url.map(|url| {
            // Resolve crossposts (starting with /r/ or /user/)
            if url.starts_with("/") {
                format!("https://reddit.com{url}")
            } else {
                url
            }
        });

        // URL parsing is pretty permissive, so sanity check to remove garbo
        if let Some(url) = &maybe_url {
            match Url::parse(url) {
                Ok(parsed) => {
                    if !["http", "https"].contains(&parsed.scheme()) || url.contains(' ') {
                        maybe_url = None;
                    }
                }
                Err(error) => {
                    tracing::warn!(%error, %url, "Valid URL failed parsing!");
                    maybe_url = None;
                }
            }
        }

        Self {
            id: SmallString::from(id),
            author: SmallString::from(author),
            score,
            title: title.to_owned(),
            created,
            body: selftext.map(ToOwned::to_owned),
            link: maybe_url,
            category: None,
        }
    }
}
