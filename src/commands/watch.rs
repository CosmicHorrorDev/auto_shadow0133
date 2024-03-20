use crate::{config, database, filter, reddit};

use std::{thread, time::Duration};

const EVENT_LOOP_SLEEP_SEC: u64 = 60;

pub fn run() -> anyhow::Result<()> {
    let mut watcher = reddit::Watcher::new();
    let db = database::Database::new()?;
    let config = config::expect_config();

    let mut num_ham = 0;
    let mut num_spam = 0;
    let mut unknown = 0;

    loop {
        let reddit::Update { fresh, expired } = watcher.update();

        db.insert_posts(expired)?;

        for post in &fresh {
            let status = filter::filter(post, &config, &db);
            match status {
                Some(filter::Status::Spam(_)) => num_spam += 1,
                Some(filter::Status::Ham(_)) => num_ham += 1,
                None => unknown += 1,
            }
            tracing::info!(num_spam, num_ham, unknown, filter_result = ?status);
        }

        thread::sleep(Duration::from_secs(EVENT_LOOP_SLEEP_SEC));
    }
}
