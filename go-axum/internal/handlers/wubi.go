package handlers

import (
	"net/http"

	"github.com/gin-gonic/gin"
	"github.com/java131313/go-axum/internal/models"
	"gorm.io/gorm"
)

type WubiHandler struct {
	db *gorm.DB
}

func NewWubiHandler(db *gorm.DB) *WubiHandler {
	return &WubiHandler{db: db}
}

type UpdateWubiCodeRequest struct {
	Character string `json:"character" binding:"required"`
	Code     string `json:"code" binding:"required"`
}

func (h *WubiHandler) GetWubiCode(c *gin.Context) {
	character := c.Param("character")

	if len(character) != 1 {
		c.JSON(http.StatusBadRequest, gin.H{"error": "Character must be a single character"})
		return
	}

	var wubiChar models.WubiCharacter
	if err := h.db.Where("character = ?", character).First(&wubiChar).Error; err != nil {
		c.JSON(http.StatusNotFound, gin.H{"error": "Character not found"})
		return
	}

	c.JSON(http.StatusOK, wubiChar)
}

func (h *WubiHandler) UpdateWubiCode(c *gin.Context) {
	var req UpdateWubiCodeRequest
	if err := c.ShouldBindJSON(&req); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": err.Error()})
		return
	}

	var wubiChar models.WubiCharacter
	result := h.db.Where("character = ?", req.Character).First(&wubiChar)

	if result.Error != nil {
		wubiChar = models.WubiCharacter{
			Character: req.Character,
			FullCode:  req.Code,
		}
		if err := h.db.Create(&wubiChar).Error; err != nil {
			c.JSON(http.StatusInternalServerError, gin.H{"error": "Failed to create wubi character"})
			return
		}
	} else {
		wubiChar.FullCode = req.Code
		if err := h.db.Save(&wubiChar).Error; err != nil {
			c.JSON(http.StatusInternalServerError, gin.H{"error": "Failed to update wubi character"})
			return
		}
	}

	c.JSON(http.StatusOK, wubiChar)
}
