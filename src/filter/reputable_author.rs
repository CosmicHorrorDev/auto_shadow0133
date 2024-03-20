use super::{Context, HamReason, Status};

// TODO: move to config
const KARMA_THRESHOLD: u16 = 3;
const NUM_POSTS_THRESHOLD: u32 = 2;

pub fn filter(Context { post, database, .. }: Context) -> Option<Status> {
    let num_posts = database
        .get_num_posts_with_author_and_min_karma(&post.author, KARMA_THRESHOLD)
        .ok()?;

    if num_posts >= NUM_POSTS_THRESHOLD {
        Some(Status::Ham(HamReason::ReputableAuthor {
            author: post.author.to_string(),
            num_reputable_posts: num_posts,
        }))
    } else {
        None
    }
}
