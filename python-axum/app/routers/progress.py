from fastapi import APIRouter, Depends, status
from sqlalchemy.orm import Session
from app.core.database import get_db
from app.models.models import UserProgress
from app.schemas.schemas import ProgressCreate, ProgressResponse

router = APIRouter(prefix="/api", tags=["progress"])


@router.post("/progress", response_model=ProgressResponse, status_code=status.HTTP_201_CREATED)
async def post_progress(progress: ProgressCreate, db: Session = Depends(get_db)):
    new_progress = UserProgress(
        user_name=progress.user_name,
        lesson_id=progress.lesson_id,
        accuracy=progress.accuracy,
        score=progress.score
    )
    db.add(new_progress)
    db.commit()
    db.refresh(new_progress)
    return new_progress
