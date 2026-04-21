from fastapi import APIRouter, Depends
from sqlalchemy.orm import Session
from typing import List
from app.core.database import get_db
from app.models.models import EnglishText
from app.schemas.schemas import EnglishTextResponse

router = APIRouter(prefix="/api", tags=["english-texts"])


@router.get("/english-texts", response_model=List[EnglishTextResponse])
async def get_english_texts(db: Session = Depends(get_db)):
    english_texts = db.query(EnglishText).order_by(EnglishText.id).all()
    return english_texts
