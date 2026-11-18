CREATE TABLE lessons (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                character_val VARCHAR(32) NOT NULL,
                code VARCHAR(32) NOT NULL,
                description TEXT NOT NULL
            );

CREATE TABLE articles (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title VARCHAR(255) NOT NULL,
                content TEXT NOT NULL,
                difficulty VARCHAR(10) DEFAULT 'medium'
            );

CREATE TABLE english_texts (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title VARCHAR(255) NOT NULL,
                content TEXT NOT NULL,
                difficulty VARCHAR(10) DEFAULT 'medium'
            );

CREATE TABLE japanese_keyboards (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                layout_name VARCHAR(100) NOT NULL UNIQUE,
                description TEXT
            );

CREATE TABLE japanese_characters (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                "character" VARCHAR(10) NOT NULL,
                romanji VARCHAR(50) NOT NULL,
                keyboard_key VARCHAR(10) NOT NULL,
                layout_id INT NOT NULL,
                FOREIGN KEY (layout_id) REFERENCES japanese_keyboards(id)
            );

CREATE TABLE japanese_texts (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title VARCHAR(255) NOT NULL,
                content TEXT NOT NULL,
                difficulty VARCHAR(10) DEFAULT 'medium',
                type VARCHAR(20) DEFAULT 'mixed'
            );

CREATE TABLE key_radicals (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                key_char VARCHAR(4) NOT NULL UNIQUE,
                radicals TEXT NOT NULL,
                formula TEXT,
                description TEXT
            );

CREATE TABLE wubi_roots (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                character_val VARCHAR(32) NOT NULL,
                code VARCHAR(32) NOT NULL,
                position VARCHAR(64) NOT NULL,
                description TEXT
            );

CREATE TABLE bopomofo_keyboards (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                layout_name VARCHAR(100) NOT NULL UNIQUE,
                description TEXT
            );

CREATE TABLE bopomofo_characters (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                "character" VARCHAR(10) NOT NULL,
                bopomofo VARCHAR(50) NOT NULL,
                keyboard_key VARCHAR(10) NOT NULL,
                layout_id INT NOT NULL,
                FOREIGN KEY (layout_id) REFERENCES bopomofo_keyboards(id)
            );

CREATE TABLE traditional_chinese_texts (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title VARCHAR(255) NOT NULL,
                content TEXT NOT NULL,
                difficulty VARCHAR(10) DEFAULT 'medium'
            );

CREATE TABLE japanese_gojuon (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                "character" VARCHAR(10) NOT NULL UNIQUE,
                romanji VARCHAR(50) NOT NULL,
                "row" VARCHAR(20) NOT NULL,
                "column" VARCHAR(20) NOT NULL,
                pronunciation VARCHAR(100) NOT NULL
            );

CREATE TABLE japanese_kanji (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                kanji VARCHAR(10) NOT NULL UNIQUE,
                reading VARCHAR(100) NOT NULL,
                meaning VARCHAR(255) NOT NULL,
                level VARCHAR(20) NOT NULL
            );

CREATE TABLE custom_articles (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title VARCHAR(255) NOT NULL,
                content TEXT NOT NULL,
                difficulty VARCHAR(10) DEFAULT 'custom'
            , lang VARCHAR(10) NOT NULL DEFAULT 'zh');

CREATE TABLE user_progress (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_name VARCHAR(64) NOT NULL,
                lesson_id INT NOT NULL,
                accuracy FLOAT NOT NULL,
                score INT NOT NULL,
                updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            );

CREATE TABLE users (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                username VARCHAR(64) NOT NULL UNIQUE,
                email VARCHAR(255) NOT NULL UNIQUE,
                password_hash VARCHAR(255) NOT NULL,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            );

CREATE TABLE wubi_characters (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                character_val VARCHAR(64) NOT NULL UNIQUE,
                simple_code VARCHAR(8) NOT NULL DEFAULT '',
                full_code VARCHAR(8) NOT NULL DEFAULT '',
                pinyin VARCHAR(64) NOT NULL DEFAULT '',
                remark VARCHAR(128) NOT NULL DEFAULT '',
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            );

