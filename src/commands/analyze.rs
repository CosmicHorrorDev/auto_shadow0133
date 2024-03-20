use std::cell::OnceCell;

use crate::{
    config, database,
    filter::{self, Status},
    types::Category,
};

pub fn run() -> anyhow::Result<()> {
    let db = database::Database::new()?;
    let config = config::expect_config();

    let posts = db.get_posts(Category::Lang, 10_000)?;
    let mut num_spam = 0;
    let mut num_ham = 0;
    let mut num_unknown = 0;

    for post in &posts {
        println!("---");
        let start = std::time::Instant::now();
        let filter_iter = filter::FilterIter::new(post, &config, &db);

        let mut first_status = None;
        for status in filter_iter.filter_map(|(_, maybe_status)| maybe_status) {
            println!("{:?}", status);
            first_status = first_status.or(Some(status));
        }
        match first_status {
            Some(Status::Spam(_)) => num_spam += 1,
            Some(Status::Ham(_)) => num_ham += 1,
            None => num_unknown += 1,
        }
        tracing::info!("Post analysis finished in {:?}", start.elapsed());
    }

    let total = (num_spam + num_ham + num_unknown) as f32;
    let percent = |num, total| num as f32 / total * 100.0;
    let percent_spam = percent(num_spam, total);
    let percent_ham = percent(num_ham, total);
    let percent_unknown = percent(num_unknown, total);
    println!(
        "Spam: {num_spam} ({percent_spam:.02}%) Ham: {num_ham} ({percent_ham:.02}%) \
        Unknown: {num_unknown} ({percent_unknown:.02}%)"
    );

    Ok(())
}
