-- Disable foreign keys temporarily to prevent drop conflicts
PRAGMA foreign_keys = OFF;

-- Drop Virtual Tables first (FTS5)
DROP TABLE IF EXISTS articles_fts;
DROP TABLE IF EXISTS articletags_fts;

-- Drop Junction / Dependent Tables
DROP TABLE IF EXISTS Follows;
DROP TABLE IF EXISTS ArticleTags;
DROP TABLE IF EXISTS FavArticles;
DROP TABLE IF EXISTS Comments;

-- Drop Base Tables
DROP TABLE IF EXISTS Articles;
DROP TABLE IF EXISTS Users;

-- Re-enable foreign key checks
PRAGMA foreign_keys = ON;