-- Add migration script here
CREATE TABLE IF NOT EXISTS user (
                id INTEGER PRIMARY KEY,
                username VARCHAR(255) NOT NULL,
                email VARCHAR(255) NOT NULL,
                new_email VARCHAR(255),
                avatar TEXT NOT NULL,
                login_code INT(6),
                tokens TEXT,
                google_oauth VARCHAR(255),
                apple_oauth VARCHAR(255),
                github_oauth VARCHAR(255),
                discord_oauth VARCHAR(255)
             );