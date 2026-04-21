from fastapi import APIRouter, Depends, HTTPException, status
from sqlalchemy.orm import Session
from typing import List
from app.core.database import get_db
from app.models.models import Lesson
from app.schemas.schemas import LessonCreate, LessonResponse

router = APIRouter(prefix="/api", tags=["lessons"])


@router.get("/lessons", response_model=List[LessonResponse])
async def get_lessons(db: Session = Depends(get_db)):
    lessons = db.query(Lesson).order_by(Lesson.id).all()
    return lessons


@router.get("/lessons/{lesson_id}", response_model=LessonResponse)
async def get_lesson(lesson_id: int, db: Session = Depends(get_db)):
    lesson = db.query(Lesson).filter(Lesson.id == lesson_id).first()
    if not lesson:
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND,
            detail="Lesson not found"
        )
    return lesson


@router.post("/lessons", response_model=LessonResponse, status_code=status.HTTP_201_CREATED)
async def create_lesson(lesson: LessonCreate, db: Session = Depends(get_db)):
    new_lesson = Lesson(
        character=lesson.character,
        code=lesson.code,
        description=lesson.description
    )
    db.add(new_lesson)
    db.commit()
    db.refresh(new_lesson)
    return new_lesson
