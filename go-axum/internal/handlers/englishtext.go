package handlers

import (
	"net/http"

	"github.com/gin-gonic/gin"
	"github.com/java131313/go-axum/internal/models"
	"gorm.io/gorm"
)

type EnglishTextHandler struct {
	db *gorm.DB
}

func NewEnglishTextHandler(db *gorm.DB) *EnglishTextHandler {
	return &EnglishTextHandler{db: db}
}

func (h *EnglishTextHandler) GetEnglishTexts(c *gin.Context) {
	var englishTexts []models.EnglishText
	if err := h.db.Order("id").Find(&englishTexts).Error; err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "Failed to fetch english texts"})
		return
	}

	if englishTexts == nil {
		englishTexts = []models.EnglishText{}
	}

	c.JSON(http.StatusOK, englishTexts)
}
