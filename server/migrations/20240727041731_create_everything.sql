-- Add migration script here
CREATE TABLE IF NOT EXISTS room (
                id INTEGER PRIMARY KEY,
                roomid VARCHAR(255) NOT NULL,
                admin_only_send BOOL NOT NULL,
                admin_only_view BOOL NOT NULL
            );

CREATE TABLE IF NOT EXISTS user (
                id INTEGER PRIMARY KEY,
                userid VARCHAR(255) NOT NULL,
                name VARCHAR(255) NOT NULL,
                owner BOOL NOT NULL,
                admin BOOL NOT NULL
            );

CREATE TABLE IF NOT EXISTS message (
                id INTEGER PRIMARY KEY,
                timestamp DATETIME NOT NULL,
                user INT NOT NULL,
                room INT NOT NULL,
                msg_type VARCHAR CHECK( msg_type IN ('text', 'attachment', 'reply', 'edit', 'reaction', 'redaction')) NOT NULL,
                msg_text TEXT,
                referencing_id INTEGER,
                emoji TEXT
            );

CREATE TABLE IF NOT EXISTS banned (
                id INTEGER PRIMARY KEY,
                userid VARCHAR(255) NOT NULL
            );