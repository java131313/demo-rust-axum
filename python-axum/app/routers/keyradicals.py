from fastapi import APIRouter, Depends, HTTPException, status
from sqlalchemy.orm import Session
from typing import List
from app.core.database import get_db
from app.models.models import KeyRadical
from app.schemas.schemas import KeyRadicalResponse

router = APIRouter(prefix="/api", tags=["key-radicals"])


@router.get("/key-radicals", response_model=List[KeyRadicalResponse])
async def get_key_radicals(db: Session = Depends(get_db)):
    key_radicals = db.query(KeyRadical).order_by(KeyRadical.id).all()
    return key_radicals


@router.get("/key-radicals/{key_char}", response_model=KeyRadicalResponse)
async def get_key_radical(key_char: str, db: Session = Depends(get_db)):
    key_radical = db.query(KeyRadical).filter(KeyRadical.key_char == key_char).first()
    if not key_radical:
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND,
            detail="Key radical not found"
        )
    return key_radical
