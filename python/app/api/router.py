from fastapi import APIRouter

from app.api.investigations import router as investigations_router

api_router = APIRouter()
api_router.include_router(investigations_router)
