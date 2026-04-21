from fastapi import APIRouter, Depends, HTTPException, status
from sqlalchemy.orm import Session
from app.core.database import get_db
from app.models.models import WubiCharacter
from app.schemas.schemas import WubiCharacterResponse, UpdateWubiCodeRequest

router = APIRouter(prefix="/api", tags=["wubi"])


@router.get("/wubi/{character}", response_model=WubiCharacterResponse)
async def get_wubi_code(character: str, db: Session = Depends(get_db)):
    if len(character) != 1:
        raise HTTPException(
            status_code=status.HTTP_400_BAD_REQUEST,
            detail="Character must be a single character"
        )

    wubi_char = db.query(WubiCharacter).filter(WubiCharacter.character == character).first()
    if not wubi_char:
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND,
            detail="Character not found"
        )

    return wubi_char


@router.put("/wubi-code", response_model=WubiCharacterResponse)
async def update_wubi_code(request: UpdateWubiCodeRequest, db: Session = Depends(get_db)):
    wubi_char = db.query(WubiCharacter).filter(WubiCharacter.character == request.character).first()

    if not wubi_char:
        wubi_char = WubiCharacter(
            character=request.character,
            full_code=request.code
        )
        db.add(wubi_char)
    else:
        wubi_char.full_code = request.code

    db.commit()
    db.refresh(wubi_char)
    return wubi_char
