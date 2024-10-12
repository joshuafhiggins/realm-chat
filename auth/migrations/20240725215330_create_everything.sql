-- Add migration script here
CREATE TABLE IF NOT EXISTS user (
                id INTEGER PRIMARY KEY,
                username VARCHAR(255) NOT NULL,
                email VARCHAR(255) NOT NULL,
                new_email VARCHAR(255),
                avatar TEXT NOT NULL,
                servers TEXT NOT NULL,
                login_code INT(6),
                tokens TEXT
             );