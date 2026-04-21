package handlers

import (
	"net/http"

	"github.com/gin-gonic/gin"
	"github.com/java131313/go-axum/internal/models"
	"gorm.io/gorm"
)

type KeyRadicalHandler struct {
	db *gorm.DB
}

func NewKeyRadicalHandler(db *gorm.DB) *KeyRadicalHandler {
	return &KeyRadicalHandler{db: db}
}

func (h *KeyRadicalHandler) GetKeyRadicals(c *gin.Context) {
	var keyRadicals []models.KeyRadical
	if err := h.db.Order("id").Find(&keyRadicals).Error; err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "Failed to fetch key radicals"})
		return
	}

	if keyRadicals == nil {
		keyRadicals = []models.KeyRadical{}
	}

	c.JSON(http.StatusOK, keyRadicals)
}

func (h *KeyRadicalHandler) GetKeyRadicalByKey(c *gin.Context) {
	keyChar := c.Param("key")

	var keyRadical models.KeyRadical
	if err := h.db.Where("key_char = ?", keyChar).First(&keyRadical).Error; err != nil {
		if err == gorm.ErrRecordNotFound {
			c.JSON(http.StatusNotFound, gin.H{"error": "Key radical not found"})
			return
		}
		c.JSON(http.StatusInternalServerError, gin.H{"error": "Failed to fetch key radical"})
		return
	}

	c.JSON(http.StatusOK, keyRadical)
}
