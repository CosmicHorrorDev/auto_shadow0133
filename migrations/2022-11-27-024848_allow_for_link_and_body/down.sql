-- START Switch back to kind+body instead of link+body. Keep the link if both
-- body and link are present
ALTER TABLE posts
ADD COLUMN kind INT NOT NULL DEFAULT 1;

UPDATE posts
SET kind = 0
WHERE link IS NOT NULL;

UPDATE posts
SET body = link
WHERE link IS NOT NULL;

ALTER TABLE posts
DROP COLUMN link;
-- END

-- START Working around `ALTER COLUMN`. Making `body` not null
ALTER TABLE posts
RENAME COLUMN body TO body_temp;

ALTER TABLE posts
ADD COLUMN body TEXT NOT NULL DEFAULT '';

UPDATE posts
SET body = body_temp;

ALTER TABLE posts
DROP COLUMN body_temp;
-- END
