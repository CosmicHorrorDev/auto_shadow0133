-- Add in a column for the post category
ALTER TABLE posts
ADD COLUMN category TEXT CHECK(category in ('lang', 'game', 'other')) DEFAULT NULL;
