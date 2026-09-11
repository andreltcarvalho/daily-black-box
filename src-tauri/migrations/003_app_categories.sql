CREATE TABLE app_categories (
 app_name TEXT PRIMARY KEY,
 category TEXT NOT NULL REFERENCES categories(id),
 updated_at INTEGER NOT NULL
);

PRAGMA user_version = 3;
