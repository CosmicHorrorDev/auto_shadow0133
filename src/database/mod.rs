use std::{env, fs, path::Path};

use crate::types::{Category, Post};

use diesel::{dsl::count, prelude::*, SqliteConnection};

mod models;
mod schema;

use models::Post as DbPost;
use schema::posts::{dsl as posts_dsl, table as posts_table};

embed_migrations!("./migrations");

pub struct Database {
    conn: SqliteConnection,
}

impl Database {
    pub fn new() -> anyhow::Result<Self> {
        // TODO: Have this default to /var/opt/auto_shadow0133/posts.db on linux
        let db_url = env::var("DATABASE_URL")?;

        tracing::info!(db_url, "Connecting to database url");
        let db_path = Path::new(&db_url);
        fs::create_dir_all(db_path.parent().expect("db must have a folder"))?;

        let conn = SqliteConnection::establish(&db_url)?;
        embedded_migrations::run(&conn)?;

        Ok(Self { conn })
    }

    pub fn insert_posts(&self, posts: Vec<Post>) -> anyhow::Result<()> {
        let posts: Vec<_> = posts.into_iter().map(DbPost::from).collect();

        // Ignore existing entries, because weird combinations of posts being removed/reapproved
        // with the program stopping and starting (which clears the debounce queue) could cause the
        // same post to be added multiple times
        diesel::insert_or_ignore_into(posts_table)
            .values(&posts)
            .execute(&self.conn)?;

        Ok(())
    }

    pub fn get_posts(&self, category: Category, limit: u32) -> anyhow::Result<Vec<Post>> {
        let posts = posts_dsl::posts
            .filter(posts_dsl::category.eq(category))
            .limit(i64::from(limit))
            .load::<DbPost>(&self.conn)?
            .into_iter()
            .map(Post::from)
            .collect();
        Ok(posts)
    }

    pub fn get_num_posts_with_author_and_min_karma(
        &self,
        author: &str,
        min_karma: u16,
    ) -> anyhow::Result<u32> {
        let num_posts: i64 = posts_dsl::posts
            .filter(posts_dsl::author.eq(author))
            .filter(posts_dsl::score.ge(i32::from(min_karma)))
            .select(count(posts_dsl::id))
            .first(&self.conn)?;
        Ok(u32::try_from(num_posts).expect("Get off the computer"))
    }
}
