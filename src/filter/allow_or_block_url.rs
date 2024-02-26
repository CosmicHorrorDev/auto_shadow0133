//! Detects good link-posts off of blessed domains

use super::{Context, HamReason, SpamReason, Status};
use crate::{config::Config, types::Token};

use url::Url;

pub fn filter(
    Context {
        post,
        config: Config { url_filters },
        ..
    }: Context,
) -> Option<Status> {
    // Filter based off the link from the post
    post.link
        .as_deref()
        .and_then(|link| {
            let url = Url::parse(link).ok()?;

            if url_filters.allow.contains(&url) {
                Some(Status::Ham(HamReason::AllowedUrl(link.to_owned())))
            } else if url_filters.block.contains(&url) {
                Some(Status::Spam(SpamReason::BlockedUrl(link.to_owned())))
            } else {
                None
            }
        })
        // Or check for in-text links
        .or_else(|| {
            let mut in_text_status = None;
            for link in post.tokens().into_iter().filter_map(|token| match token {
                Token::Url { url, .. } => Some(url),
                _ => None,
            }) {
                let Ok(url) = Url::parse(&link) else {
                    continue;
                };

                // Preference given to allowed links for in-text. Someone may post a youtube video
                // and a github link for instance
                if url_filters.allow.contains(&url) {
                    in_text_status = Some(Status::Ham(HamReason::AllowedUrl(link)));
                    break;
                } else if url_filters.block.contains(&url) {
                    in_text_status = Some(Status::Spam(SpamReason::BlockedUrl(link)));
                }
            }

            in_text_status
        })
}
