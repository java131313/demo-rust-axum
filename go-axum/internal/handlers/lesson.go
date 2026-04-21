package handlers

import (
	"net/http"
	"strconv"

	"github.com/gin-gonic/gin"
	"github.com/java131313/go-axum/internal/models"
	"gorm.io/gorm"
)

type LessonHandler struct {
	db *gorm.DB
}

func NewLessonHandler(db *gorm.DB) *LessonHandler {
	return &LessonHandler{db: db}
}

type CreateLessonRequest struct {
	Character   string `json:"character" binding:"required"`
	Code        string `json:"code" binding:"required"`
	Description string `json:"description" binding:"required"`
}

func (h *LessonHandler) GetLessons(c *gin.Context) {
	var lessons []models.Lesson
	if err := h.db.Order("id").Find(&lessons).Error; err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "Failed to fetch lessons"})
		return
	}

	if lessons == nil {
		lessons = []models.Lesson{}
	}

	c.JSON(http.StatusOK, lessons)
}

func (h *LessonHandler) GetLesson(c *gin.Context) {
	idStr := c.Param("id")
	id, err := strconv.Atoi(idStr)
	if err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "Invalid lesson ID"})
		return
	}

	var lesson models.Lesson
	if err := h.db.First(&lesson, id).Error; err != nil {
		if err == gorm.ErrRecordNotFound {
			c.JSON(http.StatusNotFound, gin.H{"error": "Lesson not found"})
			return
		}
		c.JSON(http.StatusInternalServerError, gin.H{"error": "Failed to fetch lesson"})
		return
	}

	c.JSON(http.StatusOK, lesson)
}

func (h *LessonHandler) CreateLesson(c *gin.Context) {
	var req CreateLessonRequest
	if err := c.ShouldBindJSON(&req); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": err.Error()})
		return
	}

	lesson := models.Lesson{
		Character:   req.Character,
		Code:        req.Code,
		Description: req.Description,
	}

	if err := h.db.Create(&lesson).Error; err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "Failed to create lesson"})
		return
	}

	c.JSON(http.StatusCreated, lesson)
}
