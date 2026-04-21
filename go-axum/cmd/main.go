package main

import (
	"log"
	"os"

	"github.com/gin-gonic/gin"
	"github.com/java131313/go-axum/internal/config"
	"github.com/java131313/go-axum/internal/db"
	"github.com/java131313/go-axum/internal/handlers"
	"github.com/java131313/go-axum/internal/middleware"
)

func main() {
	if err := config.LoadConfig(); err != nil {
		log.Fatalf("Failed to load config: %v", err)
	}

	database, err := db.InitDB()
	if err != nil {
		log.Fatalf("Failed to initialize database: %v", err)
	}

	if err := db.InitTables(database); err != nil {
		log.Fatalf("Failed to initialize tables: %v", err)
	}

	r := gin.Default()

	r.Use(middleware.CORSMiddleware())

	healthHandler := handlers.NewHealthHandler()
	authHandler := handlers.NewAuthHandler(database)
	wubiHandler := handlers.NewWubiHandler(database)
	lessonHandler := handlers.NewLessonHandler(database)
	articleHandler := handlers.NewArticleHandler(database)
	progressHandler := handlers.NewProgressHandler(database)
	keyRadicalHandler := handlers.NewKeyRadicalHandler(database)
	englishTextHandler := handlers.NewEnglishTextHandler(database)

	api := r.Group("/api")
	{
		api.GET("/health", healthHandler.Health)

		api.POST("/login", authHandler.Login)
		api.POST("/logout", authHandler.Logout)
		api.POST("/register", authHandler.Register)

		api.GET("/wubi/:character", wubiHandler.GetWubiCode)
		api.PUT("/wubi-code", wubiHandler.UpdateWubiCode)

		api.GET("/lessons", lessonHandler.GetLessons)
		api.GET("/lessons/:id", lessonHandler.GetLesson)
		api.POST("/lessons", lessonHandler.CreateLesson)

		api.POST("/progress", progressHandler.PostProgress)

		api.GET("/articles", articleHandler.GetArticles)
		api.GET("/articles/:id", articleHandler.GetArticle)
		api.POST("/articles", articleHandler.CreateArticle)
		api.PUT("/articles/:id", articleHandler.UpdateArticle)
		api.DELETE("/articles/:id", articleHandler.DeleteArticle)

		api.GET("/key-radicals", keyRadicalHandler.GetKeyRadicals)
		api.GET("/key-radicals/:key", keyRadicalHandler.GetKeyRadicalByKey)

		api.GET("/english-texts", englishTextHandler.GetEnglishTexts)
	}

	port := os.Getenv("PORT")
	if port == "" {
		port = "3000"
	}

	log.Printf("Server starting on port %s", port)
	if err := r.Run(":" + port); err != nil {
		log.Fatalf("Failed to start server: %v", err)
	}
}
