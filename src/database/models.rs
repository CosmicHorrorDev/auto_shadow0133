use super::schema::posts;
use crate::types::Category;

// TODO: no need to micro-optimize with this kind of stuff
use smartstring::alias::String as SmallString;
use time::OffsetDateTime;

#[derive(Insertable, Queryable)]
#[table_name = "posts"]
pub struct Post {
    pub id: String,
    pub author: String,
    pub score: i32,
    pub title: String,
    pub created: f32,
    pub body: Option<String>,
    pub link: Option<String>,
    pub category: Option<Category>,
}

impl From<crate::types::Post> for Post {
    fn from(
        crate::types::Post {
            id,
            author,
            score,
            title,
            created,
            body,
            link,
            category,
        }: crate::types::Post,
    ) -> Self {
        Self {
            id: String::from(id),
            author: String::from(author),
            score: score as i32,
            title,
            created: created.unix_timestamp() as f32,
            body,
            link,
            category,
        }
    }
}

impl From<Post> for crate::types::Post {
    fn from(post: Post) -> Self {
        let Post {
            id,
            author,
            score,
            title,
            created,
            body,
            link,
            category,
        } = post;
        Self {
            id: SmallString::from(id),
            author: SmallString::from(author),
            score: score as f64,
            title,
            created: OffsetDateTime::from_unix_timestamp(created as i64).unwrap(),
            body,
            link,
            category,
        }
    }
}
