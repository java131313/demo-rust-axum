<template>
  <div class="traditional-chinese-typing-practice">
    <div class="practice-container">
      <div class="practice-header">
        <h2>繁体中文注音打字练习</h2>
        <div class="difficulty-selector">
          <label>难度：</label>
          <a-select v-model:value="selectedDifficulty" @change="handleDifficultyChange">
            <a-select-option value="easy">简单</a-select-option>
            <a-select-option value="medium">中等</a-select-option>
            <a-select-option value="hard">困难</a-select-option>
          </a-select>
        </div>
      </div>

      <div class="text-display" ref="textDisplay">
        <div class="text-line">
          <span
            v-for="(char, index) in currentText.content"
            :key="'char-' + index"
            :class="{
              'correct': index < correctChars.length,
              'current': index === currentIndex,
              'incorrect': index < currentIndex && !correctChars[index]
            }"
          >
            {{ char }}
          </span>
        </div>
        <div class="zhuyin-line" v-if="showZhuyinHint">
          <span
            v-for="(char, index) in currentText.content"
            :key="'zhuyin-' + index"
            :class="{
              'hint-visible': index >= currentIndex && index < currentIndex + 3,
              'hint-current': index === currentIndex
            }"
          >
            {{ getZhuyinHint(char) }}
          </span>
        </div>
      </div>

      <div class="input-area">
        <a-input
          v-model:value="userInput"
          @input="handleInput"
          @keyup="handleKeyUp"
          placeholder="请输入上面的文本..."
          autofocus
        />
      </div>

      <div class="bopomofo-keyboard">
        <h3>注音键盘提示</h3>
        <div class="keyboard-grid">
          <div v-if="bopomofoChars.length === 0" class="no-data">
            加载中...
          </div>
          <div
            v-for="(char, index) in bopomofoChars"
            :key="char.id"
            class="keyboard-key"
          >
            <span class="key-char">{{ char.character }}</span>
            <span class="key-bopomofo">{{ char.bopomofo }}</span>
            <span class="key-binding">{{ char.keyboard_key }}</span>
          </div>
        </div>
      </div>

      <div class="stats">
        <div class="stat-item">
          <span class="stat-label">正确率：</span>
          <span class="stat-value">{{ accuracy }}%</span>
        </div>
        <div class="stat-item">
          <span class="stat-label">速度：</span>
          <span class="stat-value">{{ speed }} 字/分钟</span>
        </div>
        <div class="stat-item">
          <span class="stat-label">用时：</span>
          <span class="stat-value">{{ timeElapsed }} 秒</span>
        </div>
      </div>

      <div class="action-buttons">
        <a-button type="primary" @click="startPractice">开始练习</a-button>
        <a-button @click="resetPractice">重置</a-button>
        <a-button @click="toggleZhuyinHint">{{ showZhuyinHint ? '隐藏注音' : '显示注音' }}</a-button>
      </div>
    </div>
  </div>
</template>

<script>
import axios from '../api';

export default {
  name: 'TraditionalChineseTypingPractice',
  data() {
    return {
      texts: [],
      bopomofoChars: [],
      currentText: { title: '', content: '', difficulty: 'medium' },
      selectedDifficulty: 'medium',
      userInput: '',
      currentIndex: 0,
      correctChars: [],
      startTime: null,
      endTime: null,
      accuracy: 0,
      speed: 0,
      timeElapsed: 0,
      timer: null,
      showZhuyinHint: true
    };
  },
  mounted() {
    this.loadTexts();
    this.loadBopomofoChars();
  },
  beforeUnmount() {
    if (this.timer) {
      clearInterval(this.timer);
    }
  },
  methods: {
    async loadTexts() {
      try {
        const response = await axios.get('/traditional-chinese-texts');
        this.texts = response.data;
        this.selectText();
      } catch (error) {
        console.error('加载繁体中文文章失败:', error);
      }
    },
    async loadBopomofoChars() {
      try {
        console.log('开始加载注音字符');
        const response = await axios.get('/bopomofo-characters');
        console.log('注音字符加载成功:', response.data);
        this.bopomofoChars = response.data;
        console.log('bopomofoChars长度:', this.bopomofoChars.length);
      } catch (error) {
        console.error('加载注音字符失败:', error);
      }
    },
    selectText() {
      const filteredTexts = this.texts.filter(text => text.difficulty === this.selectedDifficulty);
      if (filteredTexts.length > 0) {
        const randomIndex = Math.floor(Math.random() * filteredTexts.length);
        this.currentText = filteredTexts[randomIndex];
      } else {
        this.currentText = { title: '默认练习', content: '一二人三四五六七八九十', difficulty: 'easy' };
      }
      this.resetPractice();
    },
    handleDifficultyChange() {
      this.selectText();
    },
    getZhuyinHint(char) {
      const zhuyinMap = {
        '一': '一', '二': '二', '三': '三', '四': '四', '五': '五',
        '六': '六', '七': '七', '八': '八', '九': '九', '十': '十',
        '你': 'ㄋㄧ', '好': 'ㄏㄠ', '歡': 'ㄏㄨㄢ', '迎': 'ㄧㄥ', '光': 'ㄍㄨㄤ', '臨': 'ㄌㄧㄣ',
        '請': 'ㄑㄧㄥ', '問': 'ㄨㄣ', '有': 'ㄧㄡ', '什': 'ㄕㄜ', '麼': 'ㄇㄜ', '可': 'ㄎㄜ',
        '以': 'ㄧ', '幫': 'ㄅㄤ', '您': 'ㄋㄧㄣ', '的': 'ㄉㄜ', '嗎': 'ㄇㄚ', '中': 'ㄓㄨㄥ',
        '華': 'ㄏㄨㄚ', '文': 'ㄨㄣ', '化': 'ㄏㄨㄚ', '源': 'ㄩㄢ', '遠': 'ㄩㄢ', '流': 'ㄌㄧㄡ',
        '長': 'ㄓㄤ', '博': 'ㄅㄛ', '大': 'ㄉㄚ', '精': 'ㄐㄧㄥ', '深': 'ㄕㄣ',
        '值': 'ㄓ', '得': 'ㄉㄜ', '我': 'ㄨㄛ', '們': 'ㄇㄣ', '好': 'ㄏㄠ', '學': 'ㄒㄩㄝ',
        '習': 'ㄒㄧ', '臺': 'ㄊㄞ', '灣': 'ㄨㄢ', '是': 'ㄕ', '一': 'ㄧ', '個': 'ㄍㄜ',
        '美': 'ㄇㄟ', '麗': 'ㄌㄧ', '的': 'ㄉㄜ', '島': 'ㄉㄠ', '嶼': 'ㄩ', '擁': 'ㄩㄥ',
        '有': 'ㄧㄡ', '豐': 'ㄈㄥ', '富': 'ㄈㄨ', '自': 'ㄗ', '然': 'ㄖㄢ', '資': 'ㄗ',
        '源': 'ㄩㄢ', '和': 'ㄏㄜ', '人': 'ㄖㄣ', '文': 'ㄨㄣ', '景': 'ㄐㄧㄥ', '觀': 'ㄍㄨㄢ',
        '，': '，', '。': '。', '！': '！', '？': '？'
      };
      return zhuyinMap[char] || '';
    },
    toggleZhuyinHint() {
      this.showZhuyinHint = !this.showZhuyinHint;
    },
    startPractice() {
      this.resetPractice();
      this.startTime = Date.now();
      this.timer = setInterval(() => {
        this.timeElapsed = Math.floor((Date.now() - this.startTime) / 1000);
      }, 1000);
    },
    resetPractice() {
      this.userInput = '';
      this.currentIndex = 0;
      this.correctChars = [];
      this.startTime = null;
      this.endTime = null;
      this.accuracy = 0;
      this.speed = 0;
      this.timeElapsed = 0;
      if (this.timer) {
        clearInterval(this.timer);
        this.timer = null;
      }
    },
    handleInput() {
      if (!this.startTime) {
        this.startPractice();
      }

      const input = this.userInput;
      this.currentIndex = input.length;

      this.correctChars = [];
      for (let i = 0; i < input.length; i++) {
        if (i < this.currentText.content.length) {
          this.correctChars.push(input[i] === this.currentText.content[i]);
        } else {
          this.correctChars.push(false);
        }
      }

      const correctCount = this.correctChars.filter(correct => correct).length;
      this.accuracy = input.length > 0 ? Math.round((correctCount / input.length) * 100) : 0;

      if (this.timeElapsed > 0) {
        this.speed = Math.round((input.length / this.timeElapsed) * 60);
      }

      if (input.length >= this.currentText.content.length) {
        this.endPractice();
      }
    },
    handleKeyUp(event) {
      if (event.key === 'Backspace' && this.currentIndex > 0) {
        this.currentIndex--;
        this.correctChars.pop();
      }
    },
    endPractice() {
      this.endTime = Date.now();
      if (this.timer) {
        clearInterval(this.timer);
        this.timer = null;
      }

      const correctCount = this.correctChars.filter(correct => correct).length;
      this.accuracy = Math.round((correctCount / this.currentText.content.length) * 100);
      this.speed = Math.round((this.currentText.content.length / this.timeElapsed) * 60);
    }
  }
};
</script>

<style scoped>
.traditional-chinese-typing-practice {
  padding: 20px;
  background: #f5f5f5;
  border-radius: 8px;
  min-height: 400px;
}

.practice-container {
  max-width: 900px;
  margin: 0 auto;
}

.practice-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
}

.practice-header h2 {
  margin: 0;
  color: #333;
}

.difficulty-selector {
  display: flex;
  align-items: center;
  gap: 10px;
}

.text-display {
  background: white;
  padding: 20px;
  border-radius: 8px;
  margin-bottom: 20px;
  font-size: 18px;
  line-height: 1.6;
  min-height: 100px;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
}

.text-line {
  margin-bottom: 10px;
}

.text-display span {
  margin: 0 2px;
  padding: 2px 4px;
  border-radius: 3px;
}

.text-display span.correct {
  background-color: #d4edda;
  color: #155724;
}

.text-display span.current {
  background-color: #cce7ff;
  color: #004085;
  border: 1px solid #b8daff;
}

.text-display span.incorrect {
  background-color: #f8d7da;
  color: #721c24;
}

.zhuyin-line {
  font-size: 14px;
  color: #666;
  margin-top: 5px;
  padding-top: 5px;
  border-top: 1px dashed #ddd;
}

.zhuyin-line span {
  margin: 0 2px;
  padding: 2px 4px;
  border-radius: 3px;
  display: inline-block;
  min-width: 20px;
  text-align: center;
}

.zhuyin-line span.hint-visible {
  color: #1890ff;
  font-weight: 500;
}

.zhuyin-line span.hint-current {
  color: #f5222d;
  font-weight: 700;
  font-size: 16px;
}

.bopomofo-keyboard {
  background: white;
  padding: 15px;
  border-radius: 8px;
  margin-bottom: 20px;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
  display: block !important;
  visibility: visible !important;
  opacity: 1 !important;
  height: auto !important;
  overflow: visible !important;
}

.bopomofo-keyboard h3 {
  margin: 0 0 15px 0;
  color: #333;
  font-size: 16px;
}

.keyboard-grid {
  display: grid;
  grid-template-columns: repeat(10, 1fr);
  gap: 8px;
  display: grid !important;
}

.keyboard-key {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 8px 4px;
  background: #f0f0f0;
  border-radius: 4px;
  border: 1px solid #d9d9d9;
  min-width: 50px;
  display: flex !important;
  visibility: visible !important;
  opacity: 1 !important;
}

.keyboard-key .key-char {
  font-size: 18px;
  font-weight: bold;
  color: #333;
}

.keyboard-key .key-bopomofo {
  font-size: 12px;
  color: #666;
  margin-top: 2px;
}

.keyboard-key .key-binding {
  font-size: 11px;
  color: #999;
  margin-top: 2px;
  background: #e0e0e0;
  padding: 1px 4px;
  border-radius: 2px;
}

.input-area {
  margin-bottom: 20px;
}

.input-area input {
  width: 100%;
  font-size: 16px;
  padding: 10px;
  border-radius: 4px;
  border: 1px solid #ddd;
}

.stats {
  display: flex;
  gap: 30px;
  margin-bottom: 20px;
  padding: 15px;
  background: white;
  border-radius: 8px;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
}

.stat-item {
  display: flex;
  align-items: center;
  gap: 5px;
}

.stat-label {
  font-weight: 600;
  color: #666;
}

.stat-value {
  font-weight: 700;
  color: #333;
}

.action-buttons {
  display: flex;
  gap: 10px;
  justify-content: center;
}

.action-buttons button {
  padding: 8px 16px;
  border-radius: 4px;
  font-size: 14px;
}

.action-buttons button[type="primary"] {
  background-color: #1890ff;
  border-color: #1890ff;
  color: white;
}

.action-buttons button[type="primary"]:hover {
  background-color: #40a9ff;
  border-color: #40a9ff;
}
</style>
