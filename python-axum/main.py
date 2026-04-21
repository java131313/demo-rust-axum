from fastapi import FastAPI
from fastapi.middleware.cors import CORSMiddleware
from app.core.config import get_settings
from app.core.database import engine, Base
from app.utils.db_init import initialize_data
from app.routers import (
    auth_router,
    wubi_router,
    lessons_router,
    articles_router,
    progress_router,
    key_radicals_router,
    english_texts_router,
)

Base.metadata.create_all(bind=engine)

app = FastAPI(
    title="五笔打字练习API",
    description="五笔打字练习系统的后端API",
    version="1.0.0"
)

app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

app.include_router(auth_router)
app.include_router(wubi_router)
app.include_router(lessons_router)
app.include_router(articles_router)
app.include_router(progress_router)
app.include_router(key_radicals_router)
app.include_router(english_texts_router)


@app.on_event("startup")
async def startup_event():
    initialize_data()


@app.get("/api/health")
async def health_check():
    return {"status": "ok"}


@app.get("/")
async def root():
    return {"message": "五笔打字练习API"}


if __name__ == "__main__":
    import uvicorn
    settings = get_settings()
    uvicorn.run("main:app", host="0.0.0.0", port=settings.port, reload=True)
