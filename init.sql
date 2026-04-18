-- PostgreSQL初始化脚本
-- 此脚本会在PostgreSQL容器首次启动时自动执行

-- 创建五笔字符表
CREATE TABLE IF NOT EXISTS wubi_characters (
    id SERIAL PRIMARY KEY,
    character_val VARCHAR(4) NOT NULL UNIQUE,
    wubi_code VARCHAR(8) NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- 创建用户表
CREATE TABLE IF NOT EXISTS users (
    id SERIAL PRIMARY KEY,
    username VARCHAR(64) NOT NULL UNIQUE,
    email VARCHAR(255) NOT NULL UNIQUE,
    password_hash VARCHAR(255) NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- 创建课程表
CREATE TABLE IF NOT EXISTS lessons (
    id SERIAL PRIMARY KEY,
    character_val VARCHAR(32) NOT NULL,
    code VARCHAR(32) NOT NULL,
    description TEXT NOT NULL
);

-- 创建用户进度表
CREATE TABLE IF NOT EXISTS user_progress (
    id SERIAL PRIMARY KEY,
    user_name VARCHAR(64) NOT NULL,
    lesson_id INT NOT NULL,
    accuracy FLOAT NOT NULL,
    score INT NOT NULL,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- 创建文章表
CREATE TABLE IF NOT EXISTS articles (
    id SERIAL PRIMARY KEY,
    title VARCHAR(255) NOT NULL,
    content TEXT NOT NULL,
    difficulty VARCHAR(10) DEFAULT 'medium'
);

-- 创建五笔字根表
CREATE TABLE IF NOT EXISTS wubi_roots (
    id SERIAL PRIMARY KEY,
    character_val VARCHAR(32) NOT NULL,
    code VARCHAR(32) NOT NULL,
    position VARCHAR(64) NOT NULL,
    description TEXT
);

-- 创建索引
CREATE INDEX IF NOT EXISTS idx_wubi_character ON wubi_characters(character_val);
CREATE INDEX IF NOT EXISTS idx_wubi_code ON wubi_characters(wubi_code);
CREATE INDEX IF NOT EXISTS idx_users_username ON users(username);

-- 插入一些初始数据
INSERT INTO wubi_characters (character_val, wubi_code) VALUES 
('一', 'GGLL'),
('的', 'RQYY'),
('是', 'JGHU'),
('在', 'DFD'),
('不', 'GII'),
('了', 'BNH'),
('有', 'DEF'),
('和', 'TKG'),
('人', 'W'),
('这', 'YPI'),
('中', 'KHK'),
('大', 'DDDD'),
('为', 'YLYI'),
('上', 'HHGG'),
('个', 'WHJ'),
('国', 'LGYI'),
('我', 'TRNT'),
('以', 'NYWY'),
('要', 'SVF'),
('他', 'WBN')
ON CONFLICT (character_val) DO NOTHING;

SELECT 'Database initialization completed!' AS message;
