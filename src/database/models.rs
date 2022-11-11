use super::schema::posts;

#[derive(Insertable)]
#[table_name = "posts"]
pub struct Post {
    pub id: String,
    pub author: String,
    pub score: i32,
    pub title: String,
    pub created: f32,
    pub kind: i32,
    pub body: String,
}

impl From<crate::types::Post> for Post {
    fn from(
        crate::types::Post {
            id,
            author,
            score,
            title,
            created,
            kind,
            body,
        }: crate::types::Post,
    ) -> Self {
        Self {
            id: String::from(id),
            author: String::from(author),
            score: score as i32,
            title,
            created: created.unix_timestamp() as f32,
            kind: i32::from(kind),
            body,
        }
    }
}
