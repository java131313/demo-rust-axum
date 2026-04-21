from pydantic import BaseModel
from typing import Optional
from datetime import datetime


class UserBase(BaseModel):
    username: str
    email: str


class UserCreate(UserBase):
    password: str


class UserResponse(UserBase):
    id: int
    created_at: Optional[datetime] = None

    class Config:
        from_attributes = True


class LoginRequest(BaseModel):
    username: str
    password: str


class Token(BaseModel):
    access_token: str
    token_type: str = "bearer"


class LoginResponse(BaseModel):
    access_token: str
    user: UserResponse


class LessonBase(BaseModel):
    character: str
    code: str
    description: str


class LessonCreate(LessonBase):
    pass


class LessonResponse(LessonBase):
    id: int

    class Config:
        from_attributes = True


class ArticleBase(BaseModel):
    title: str
    content: str
    difficulty: str = "medium"


class ArticleCreate(ArticleBase):
    pass


class ArticleUpdate(ArticleBase):
    pass


class ArticleResponse(ArticleBase):
    id: int

    class Config:
        from_attributes = True


class WubiCharacterBase(BaseModel):
    character: str


class WubiCharacterResponse(BaseModel):
    id: int
    character: str
    simple_code: str
    full_code: str
    pinyin: str
    remark: str

    class Config:
        from_attributes = True


class UpdateWubiCodeRequest(BaseModel):
    character: str
    code: str


class WubiRootBase(BaseModel):
    character: str
    code: str
    position: str
    description: Optional[str] = None


class WubiRootCreate(WubiRootBase):
    pass


class WubiRootResponse(WubiRootBase):
    id: int

    class Config:
        from_attributes = True


class ProgressBase(BaseModel):
    user_name: str
    lesson_id: int
    accuracy: float
    score: int


class ProgressCreate(ProgressBase):
    pass


class ProgressResponse(ProgressBase):
    id: int

    class Config:
        from_attributes = True


class KeyRadicalResponse(BaseModel):
    id: int
    key_char: str
    radicals: str
    formula: str
    description: str

    class Config:
        from_attributes = True


class EnglishTextResponse(BaseModel):
    id: int
    title: str
    content: str
    difficulty: str

    class Config:
        from_attributes = True
