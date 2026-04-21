from fastapi import APIRouter, Depends, HTTPException, status
from sqlalchemy.orm import Session
from typing import List
from app.core.database import get_db
from app.models.models import Article
from app.schemas.schemas import ArticleCreate, ArticleUpdate, ArticleResponse

router = APIRouter(prefix="/api", tags=["articles"])


@router.get("/articles", response_model=List[ArticleResponse])
async def get_articles(db: Session = Depends(get_db)):
    articles = db.query(Article).order_by(Article.id).all()
    return articles


@router.get("/articles/{article_id}", response_model=ArticleResponse)
async def get_article(article_id: int, db: Session = Depends(get_db)):
    article = db.query(Article).filter(Article.id == article_id).first()
    if not article:
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND,
            detail="Article not found"
        )
    return article


@router.post("/articles", response_model=ArticleResponse, status_code=status.HTTP_201_CREATED)
async def create_article(article: ArticleCreate, db: Session = Depends(get_db)):
    new_article = Article(
        title=article.title,
        content=article.content,
        difficulty=article.difficulty
    )
    db.add(new_article)
    db.commit()
    db.refresh(new_article)
    return new_article


@router.put("/articles/{article_id}", response_model=ArticleResponse)
async def update_article(article_id: int, article: ArticleUpdate, db: Session = Depends(get_db)):
    existing_article = db.query(Article).filter(Article.id == article_id).first()
    if not existing_article:
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND,
            detail="Article not found"
        )

    existing_article.title = article.title
    existing_article.content = article.content
    existing_article.difficulty = article.difficulty

    db.commit()
    db.refresh(existing_article)
    return existing_article


@router.delete("/articles/{article_id}", status_code=status.HTTP_204_NO_CONTENT)
async def delete_article(article_id: int, db: Session = Depends(get_db)):
    article = db.query(Article).filter(Article.id == article_id).first()
    if not article:
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND,
            detail="Article not found"
        )

    db.delete(article)
    db.commit()
    return None
