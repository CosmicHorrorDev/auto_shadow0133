-- Posts can have both a link and a body

-- START Working around no `ALTER COLUMN`. Making `body` nullable
ALTER TABLE posts
RENAME COLUMN body TO body_temp;

ALTER TABLE posts
ADD COLUMN body TEXT NULL;

UPDATE posts
SET body = body_temp;

ALTER TABLE posts
DROP COLUMN body_temp;
-- END

ALTER TABLE posts
ADD COLUMN link TEXT;

UPDATE posts
SET
    link = body,
    body = NULL
WHERE kind = 0;

ALTER TABLE posts
DROP COLUMN kind;
