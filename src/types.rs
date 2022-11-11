use std::{cmp::Ordering, fmt};

use crate::utils;

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
    pub kind: PostKind,
    pub body: String,
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
            kind,
            body,
        } = &self;

        let mut debug_struct = f.debug_struct("Post");
        debug_struct.field("id", id);
        debug_struct.field("author", author);
        debug_struct.field("score", score);

        let truncated_title = utils::truncate_str(title, DEBUG_FIELD_TRUNCATE_LEN);
        debug_struct.field("title", &truncated_title);

        debug_struct.field("created", &created.format(&Rfc3339).unwrap());
        debug_struct.field("kind", kind);

        let truncated_body = utils::truncate_str(body, DEBUG_FIELD_TRUNCATE_LEN);
        debug_struct.field("body", &truncated_body);

        debug_struct.finish()
    }
}

#[derive(Clone, Debug)]
pub enum PostKind {
    Link,
    Text,
}

impl From<i32> for PostKind {
    fn from(num: i32) -> Self {
        match num {
            0 => Self::Link,
            1 => Self::Text,
            _ => panic!("Got unexpected i32 for kind"),
        }
    }
}

impl From<PostKind> for i32 {
    fn from(kind: PostKind) -> Self {
        match kind {
            PostKind::Link => 0,
            PostKind::Text => 1,
        }
    }
}
