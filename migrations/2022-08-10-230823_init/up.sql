CREATE TABLE posts (
    id TEXT PRIMARY KEY,
    author TEXT NOT NULL,
    score INT NOT NULL,
    title TEXT NOT NULL,
    created REAL NOT NULL,
    kind INT NOT NULL,
    body TEXT NOT NULL DEFAULT ''
);
