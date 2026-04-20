#!/bin/bash
set -e

PROJECT_DIR="$(cd "$(dirname "$0")" && pwd)"
SRC_DIR="$PROJECT_DIR/java-swing/src/main/java"
BUILD_DIR="$PROJECT_DIR/java-swing/build"
MAIN_CLASS="com.wubi.client.WubiSwingApp"

echo "Cleaning build directory..."
rm -rf "$BUILD_DIR"
mkdir -p "$BUILD_DIR"

echo "Compiling Java sources..."
javac -d "$BUILD_DIR" \
    "$SRC_DIR/com/wubi/client/model/User.java" \
    "$SRC_DIR/com/wubi/client/model/WubiCharacter.java" \
    "$SRC_DIR/com/wubi/client/model/KeyRadical.java" \
    "$SRC_DIR/com/wubi/client/model/Lesson.java" \
    "$SRC_DIR/com/wubi/client/api/WubiApiClient.java" \
    "$SRC_DIR/com/wubi/client/ui/LoginDialog.java" \
    "$SRC_DIR/com/wubi/client/ui/WubiLookupPanel.java" \
    "$SRC_DIR/com/wubi/client/ui/WubiKeyboardPanel.java" \
    "$SRC_DIR/com/wubi/client/ui/WubiTypingPanel.java" \
    "$SRC_DIR/com/wubi/client/ui/EnglishTypingPanel.java" \
    "$SRC_DIR/com/wubi/client/ui/PinyinTypingPanel.java" \
    "$SRC_DIR/com/wubi/client/ui/MainWindow.java" \
    "$SRC_DIR/com/wubi/client/WubiSwingApp.java"

echo "Creating JAR..."
jar cfe "$PROJECT_DIR/wubi-swing-client.jar" "$MAIN_CLASS" -C "$BUILD_DIR" .

echo "Build successful!"
echo "Run with: java -jar $PROJECT_DIR/wubi-swing-client.jar"
