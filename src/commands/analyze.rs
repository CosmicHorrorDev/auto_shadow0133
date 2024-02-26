use crate::{config, database, filter, types::Category};

pub fn run() -> anyhow::Result<()> {
    let db = database::Database::new()?;
    let config = config::Config::new()?;

    let posts = db.get_posts(Category::Lang, 10_000)?;

    for post in &posts {
        println!("---");
        let filter_iter = filter::FilterIter::new(post, &config, &db);

        for status in filter_iter.filter_map(|(_, maybe_status)| maybe_status) {
            println!("{:?}", status);
        }
    }

    dbg!(posts.len());

    Ok(())
}
