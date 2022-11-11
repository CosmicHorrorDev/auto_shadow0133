use std::{env, fs, path::Path};

use diesel::{prelude::*, SqliteConnection};

mod models;
mod schema;

embed_migrations!("./migrations");

pub struct Database {
    conn: SqliteConnection,
}

impl Database {
    pub fn new() -> anyhow::Result<Self> {
        let _ = dotenv::dotenv();

        let db_url = env::var("DATABASE_URL")?;

        let db_path = Path::new(&db_url);
        fs::create_dir_all(db_path.parent().expect("db must have a folder"))?;

        let conn = SqliteConnection::establish(&db_url)?;
        embedded_migrations::run(&conn)?;

        Ok(Self { conn })
    }

    pub fn insert_posts(&self, posts: Vec<crate::types::Post>) -> anyhow::Result<()> {
        let posts: Vec<_> = posts.into_iter().map(models::Post::from).collect();

        // Ignore existing entries, because weird combinations of posts being removed/reapproved
        // with the program stopping and starting (which clears the debounce queue) could cause the
        // same post to be added multiple times
        diesel::insert_or_ignore_into(schema::posts::table)
            .values(&posts)
            .execute(&self.conn)?;

        Ok(())
    }
}
