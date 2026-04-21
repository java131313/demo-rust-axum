package models

import (
	"time"
)

type User struct {
	ID           uint      `json:"id" gorm:"primaryKey"`
	Username     string    `json:"username" gorm:"uniqueIndex;size:64;not null"`
	Email        string    `json:"email" gorm:"uniqueIndex;size:255;not null"`
	PasswordHash string    `json:"-" gorm:"size:255;not null"`
	CreatedAt    time.Time `json:"created_at"`
}

type Lesson struct {
	ID          uint   `json:"id" gorm:"primaryKey"`
	Character   string `json:"character" gorm:"size:32;not null"`
	Code        string `json:"code" gorm:"size:32;not null"`
	Description string `json:"description" gorm:"type:text;not null"`
}

type Article struct {
	ID         uint   `json:"id" gorm:"primaryKey"`
	Title      string `json:"title" gorm:"size:255;not null"`
	Content    string `json:"content" gorm:"type:text;not null"`
	Difficulty string `json:"difficulty" gorm:"size:10;default:'medium'"`
}

type WubiCharacter struct {
	ID          uint   `json:"id" gorm:"primaryKey"`
	Character   string `json:"character" gorm:"uniqueIndex;size:32;not null"`
	SimpleCode  string `json:"simple_code" gorm:"size:8;default:''"`
	FullCode    string `json:"full_code" gorm:"size:8;default:''"`
	Pinyin      string `json:"pinyin" gorm:"size:32;default:''"`
	Remark      string `json:"remark" gorm:"size:128;default:''"`
}

type WubiRoot struct {
	ID          uint   `json:"id" gorm:"primaryKey"`
	Character   string `json:"character" gorm:"size:32;not null"`
	Code        string `json:"code" gorm:"size:32;not null"`
	Position    string `json:"position" gorm:"size:64;not null"`
	Description string `json:"description" gorm:"type:text"`
}

type UserProgress struct {
	ID        uint      `json:"id" gorm:"primaryKey"`
	UserName  string    `json:"user_name" gorm:"size:64;not null"`
	LessonID  uint      `json:"lesson_id" gorm:"not null"`
	Accuracy  float32   `json:"accuracy" gorm:"not null"`
	Score     int       `json:"score" gorm:"not null"`
	UpdatedAt time.Time `json:"updated_at"`
}

type KeyRadical struct {
	ID          uint   `json:"id" gorm:"primaryKey"`
	KeyChar     string `json:"key_char" gorm:"uniqueIndex;size:4;not null"`
	Radicals    string `json:"radicals" gorm:"type:text;not null"`
	Formula     string `json:"formula" gorm:"type:text"`
	Description string `json:"description" gorm:"type:text"`
}

type EnglishText struct {
	ID         uint   `json:"id" gorm:"primaryKey"`
	Title      string `json:"title" gorm:"size:128;not null"`
	Content    string `json:"content" gorm:"type:text;not null"`
	Difficulty string `json:"difficulty" gorm:"size:16;not null"`
}
