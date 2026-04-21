package handlers

import (
	"net/http"

	"github.com/gin-gonic/gin"
	"github.com/java131313/go-axum/internal/models"
	"gorm.io/gorm"
)

type ProgressHandler struct {
	db *gorm.DB
}

func NewProgressHandler(db *gorm.DB) *ProgressHandler {
	return &ProgressHandler{db: db}
}

type PostProgressRequest struct {
	UserName string  `json:"user_name" binding:"required"`
	LessonID uint    `json:"lesson_id" binding:"required"`
	Accuracy float32 `json:"accuracy" binding:"required"`
	Score    int     `json:"score" binding:"required"`
}

func (h *ProgressHandler) PostProgress(c *gin.Context) {
	var req PostProgressRequest
	if err := c.ShouldBindJSON(&req); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": err.Error()})
		return
	}

	progress := models.UserProgress{
		UserName: req.UserName,
		LessonID: req.LessonID,
		Accuracy: req.Accuracy,
		Score:    req.Score,
	}

	if err := h.db.Create(&progress).Error; err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "Failed to save progress"})
		return
	}

	c.JSON(http.StatusCreated, progress)
}
