from sqlalchemy import Column, Integer, String, Text, Float, DateTime
from sqlalchemy.sql import func
from app.core.database import Base


class User(Base):
    __tablename__ = "users"

    id = Column(Integer, primary_key=True, index=True)
    username = Column(String(64), unique=True, nullable=False, index=True)
    email = Column(String(255), unique=True, nullable=False, index=True)
    password_hash = Column(String(255), nullable=False)
    created_at = Column(DateTime, server_default=func.now())


class Lesson(Base):
    __tablename__ = "lessons"

    id = Column(Integer, primary_key=True, index=True)
    character = Column(String(32), nullable=False)
    code = Column(String(32), nullable=False)
    description = Column(Text, nullable=False)


class Article(Base):
    __tablename__ = "articles"

    id = Column(Integer, primary_key=True, index=True)
    title = Column(String(255), nullable=False)
    content = Column(Text, nullable=False)
    difficulty = Column(String(10), default="medium")


class WubiCharacter(Base):
    __tablename__ = "wubi_characters"

    id = Column(Integer, primary_key=True, index=True)
    character = Column(String(32), unique=True, nullable=False, index=True)
    simple_code = Column(String(8), default="")
    full_code = Column(String(8), default="")
    pinyin = Column(String(32), default="")
    remark = Column(String(128), default="")


class WubiRoot(Base):
    __tablename__ = "wubi_roots"

    id = Column(Integer, primary_key=True, index=True)
    character = Column(String(32), nullable=False)
    code = Column(String(32), nullable=False)
    position = Column(String(64), nullable=False)
    description = Column(Text)


class UserProgress(Base):
    __tablename__ = "user_progress"

    id = Column(Integer, primary_key=True, index=True)
    user_name = Column(String(64), nullable=False)
    lesson_id = Column(Integer, nullable=False)
    accuracy = Column(Float, nullable=False)
    score = Column(Integer, nullable=False)
    updated_at = Column(DateTime, server_default=func.now(), onupdate=func.now())


class KeyRadical(Base):
    __tablename__ = "key_radicals"

    id = Column(Integer, primary_key=True, index=True)
    key_char = Column(String(4), unique=True, nullable=False, index=True)
    radicals = Column(Text, nullable=False)
    formula = Column(Text)
    description = Column(Text)


class EnglishText(Base):
    __tablename__ = "english_texts"

    id = Column(Integer, primary_key=True, index=True)
    title = Column(String(128), nullable=False)
    content = Column(Text, nullable=False)
    difficulty = Column(String(16), nullable=False)
