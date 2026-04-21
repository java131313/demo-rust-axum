package db

import (
	"fmt"
	"log"

	"github.com/java131313/go-axum/internal/config"
	"github.com/java131313/go-axum/internal/models"
	"gorm.io/driver/mysql"
	"gorm.io/gorm"
)

var DB *gorm.DB

func InitDB() (*gorm.DB, error) {
	cfg := config.AppConfig
	dsn := fmt.Sprintf("%s:%s@tcp(%s:%s)/%s?charset=utf8mb4&parseTime=True&loc=Local",
		cfg.Database.Username,
		cfg.Database.Password,
		cfg.Database.Host,
		cfg.Database.Port,
		cfg.Database.Database,
	)

	db, err := gorm.Open(mysql.Open(dsn), &gorm.Config{})
	if err != nil {
		return nil, fmt.Errorf("failed to connect to database: %w", err)
	}

	DB = db
	log.Println("Database connection established")
	return db, nil
}

func InitTables(db *gorm.DB) error {
	// 检查并创建不存在的表
	tables := []interface{}{
		&models.User{},
		&models.Lesson{},
		&models.Article{},
		&models.WubiRoot{},
		&models.UserProgress{},
		&models.KeyRadical{},
		&models.EnglishText{},
	}

	for _, table := range tables {
		if err := db.AutoMigrate(table); err != nil {
			log.Printf("Warning: failed to migrate table %T: %v", table, err)
		}
	}

	// 特殊处理 WubiCharacter 表，避免重复键错误
	if !db.Migrator().HasTable(&models.WubiCharacter{}) {
		if err := db.AutoMigrate(&models.WubiCharacter{}); err != nil {
			log.Printf("Warning: failed to migrate WubiCharacter table: %v", err)
		}
	}

	if err := initKeyRadicals(db); err != nil {
		log.Printf("Warning: failed to initialize key radicals: %v", err)
	}

	if err := initEnglishTexts(db); err != nil {
		log.Printf("Warning: failed to initialize english texts: %v", err)
	}

	log.Println("Database tables initialized")
	return nil
}

func initKeyRadicals(db *gorm.DB) error {
	var count int64
	db.Model(&models.KeyRadical{}).Count(&count)
	if count > 0 {
		return nil
	}

	keyRadicals := []models.KeyRadical{
		{KeyChar: "g", Radicals: "王、一、五、戋", Formula: "王旁青头戋（兼）五一", Description: "G区横区第一键，包含横笔和戈字根"},
		{KeyChar: "f", Radicals: "土、士、二、干、十、寸、雨", Formula: "土士二干十寸雨", Description: "F区横区第二键，包含土字根"},
		{KeyChar: "d", Radicals: "大、犬、三、古、石、厂", Formula: "大犬三（古）石厂", Description: "D区横区第三键，包含大字根"},
		{KeyChar: "s", Radicals: "木、丁、西", Formula: "木丁西", Description: "S区横区第四键，包含木字根"},
		{KeyChar: "a", Radicals: "工、戈、艹、七、廿", Formula: "工戈草头右框七", Description: "A区横区第五键，包含工字根"},
		{KeyChar: "h", Radicals: "目、止、卜、虍、上", Formula: "目具上止卜虎皮", Description: "H区竖区第一键，包含目字根"},
		{KeyChar: "j", Radicals: "日、早、虫、刂、竖", Formula: "日早两竖与虫依", Description: "J区竖区第二键，包含日字根"},
		{KeyChar: "k", Radicals: "口、川", Formula: "口与川，字根稀", Description: "K区竖区第三键，包含口字根"},
		{KeyChar: "l", Radicals: "田、甲、四、车、囗", Formula: "田甲方框四车里", Description: "L区竖区第四键，包含田字根"},
		{KeyChar: "m", Radicals: "山、由、贝、几", Formula: "山由贝，下框几", Description: "M区竖区第五键，包含山字根"},
		{KeyChar: "t", Radicals: "禾、竹、丿、彳、攵", Formula: "禾竹一撇双人立", Description: "T区撇区第一键，包含禾字根"},
		{KeyChar: "r", Radicals: "白、手、斤、牛", Formula: "白手看头三二斤", Description: "R区撇区第二键，包含白字根"},
		{KeyChar: "e", Radicals: "舟、用、月、豕、衣", Formula: "舟用乃月豕（家）衣", Description: "E区撇区第三键，包含月字根"},
		{KeyChar: "w", Radicals: "人、八、亻", Formula: "人八登头单人几", Description: "W区撇区第四键，包含人字根"},
		{KeyChar: "q", Radicals: "金、饣、勹、儿、夕", Formula: "金勺缺点无尾鱼，犬旁留叉", Description: "Q区撇区第五键，包含金字根"},
		{KeyChar: "y", Radicals: "言、文、方、广、丶", Formula: "言文方广在四一，高头一捺谁人去", Description: "Y区捺区第一键，包含言字根"},
		{KeyChar: "u", Radicals: "立、辛、六、门、疒", Formula: "立辛两点六门疒（病）", Description: "U区捺区第二键，包含立字根"},
		{KeyChar: "i", Radicals: "氵（三点水）、小", Formula: "水旁兴头小倒立", Description: "I区捺区第三键，包含水字根"},
		{KeyChar: "o", Radicals: "火、米、灬", Formula: "火业头，四点米", Description: "O区捺区第四键，包含火字根"},
		{KeyChar: "p", Radicals: "之、宀（宝盖）、冖、礻、衤", Formula: "之字军盖建道底，摘礻衤", Description: "P区捺区第五键，包含之字根"},
		{KeyChar: "n", Radicals: "已、己、巳、尸、心、羽", Formula: "已半巳满不出己，左框折尸心和羽", Description: "N区折区第一键，包含已字根"},
		{KeyChar: "b", Radicals: "子、耳、了、也、卩", Formula: "子耳了也框向上", Description: "B区折区第二键，包含子字根"},
		{KeyChar: "v", Radicals: "女、刀、九、臼", Formula: "女刀九臼山朝西", Description: "V区折区第三键，包含女字根"},
		{KeyChar: "c", Radicals: "又、巴、马、厶", Formula: "又巴马，丢矢矣", Description: "C区折区第四键，包含又字根"},
		{KeyChar: "x", Radicals: "幺、母、弓、匕", Formula: "慈母无心弓和匕，幼无力", Description: "X区折区第五键，包含丝字根"},
	}

	for _, kr := range keyRadicals {
		if err := db.Create(&kr).Error; err != nil {
			return fmt.Errorf("failed to create key radical %s: %w", kr.KeyChar, err)
		}
	}

	log.Printf("Initialized %d key radicals", len(keyRadicals))
	return nil
}

func initEnglishTexts(db *gorm.DB) error {
	var count int64
	db.Model(&models.EnglishText{}).Count(&count)
	if count > 0 {
		return nil
	}

	englishTexts := []models.EnglishText{
		{Title: "基础练习", Content: "the quick brown fox jumps over the lazy dog", Difficulty: "easy"},
		{Title: "常用句子", Content: "hello world this is a typing practice text for english learning", Difficulty: "easy"},
		{Title: "进阶练习", Content: "practice makes perfect keep typing to improve your speed and accuracy", Difficulty: "medium"},
		{Title: "高级练习", Content: "the five boxing wizards jump quickly at dawn every single day", Difficulty: "hard"},
	}

	for _, et := range englishTexts {
		if err := db.Create(&et).Error; err != nil {
			return fmt.Errorf("failed to create english text %s: %w", et.Title, err)
		}
	}

	log.Printf("Initialized %d english texts", len(englishTexts))
	return nil
}
