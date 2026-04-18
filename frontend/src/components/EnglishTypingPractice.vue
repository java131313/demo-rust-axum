<template>
  <div class="english-practice-container">
    <a-card class="practice-card">
      <template #title>
        <span>英语打字练习</span>
      </template>
      
      <div class="stats-container">
        <a-row :gutter="16">
          <a-col :span="8">
            <a-statistic title="时间" :value="formatTime(elapsedTime)" />
          </a-col>
          <a-col :span="8">
            <a-statistic title="准确率" :value="accuracy" suffix="%" />
          </a-col>
          <a-col :span="8">
            <a-statistic title="速度" :value="speed" suffix="词/分钟" />
          </a-col>
        </a-row>
      </div>
      
      <div class="mode-control">
        <a-space>
          <span>练习模式：</span>
          <a-radio-group v-model:value="practiceMode">
            <a-radio-button value="manual">手动练习</a-radio-button>
            <a-radio-button value="auto">自动演示</a-radio-button>
          </a-radio-group>
        </a-space>
        
        <a-space v-if="practiceMode === 'auto'" style="margin-top: 12px;">
          <span>打字速度：</span>
          <a-select v-model:value="typingSpeed" style="width: 120px;">
            <a-select-option :value="200">慢速</a-select-option>
            <a-select-option :value="100">正常</a-select-option>
            <a-select-option :value="50">快速</a-select-option>
          </a-select>
        </a-space>
      </div>

      <div class="article-selection">
        <a-select 
          v-model:value="selectedTextId" 
          placeholder="选择练习文章"
          style="width: 100%; margin-bottom: 16px;"
          @change="handleTextChange"
        >
          <a-select-option 
            v-for="text in englishTexts" 
            :key="text.id" 
            :value="text.id"
          >
            {{ text.title }} ({{ text.difficulty }})
          </a-select-option>
        </a-select>
      </div>
      
      <div class="original-text" v-if="currentText">
        <div class="text-content">
          <span
            v-for="(char, index) in currentText.content.split('')"
            :key="index"
            :class="getCharClass(index)"
          >
            {{ char }}
          </span>
        </div>
      </div>
      
      <div class="current-char-hint" v-if="currentChar">
        <a-alert
          message="当前需要输入"
          :description="字符"
          type="info"
          show-icon
        />
      </div>

      <div class="input-section">
        <a-textarea
          v-if="practiceMode === 'manual'"
          v-model:value="userInput"
          placeholder="请在此输入英文进行练习..."
          :auto-size="{ minRows: 4, maxRows: 6 }"
          @focus="startTypingPractice"
          @input="handleUserInput"
          style="margin-top: 16px;"
        />
        <div v-else class="auto-display">
          <a-textarea
            v-model:value="userInput"
            placeholder="自动演示中..."
            :auto-size="{ minRows: 4, maxRows: 6 }"
            disabled
            style="margin-top: 16px;"
          />
        </div>
      </div>
      
      <VirtualKeyboard :activeKey="currentActiveKey" />
      
      <div class="control-buttons">
        <a-space>
          <a-button @click="resetPractice" type="primary" danger>
            重新开始
          </a-button>
          <a-button 
            v-if="practiceMode === 'auto' && !isAutoTyping" 
            @click="startAutoTyping" 
            type="primary"
          >
            开始演示
          </a-button>
          <a-button 
            v-if="practiceMode === 'auto' && isAutoTyping" 
            @click="pauseAutoTyping" 
            type="primary"
          >
            暂停
          </a-button>
          <a-button 
            v-if="practiceMode === 'auto' && isAutoTyping" 
            @click="resumeAutoTyping" 
            type="primary"
          >
            继续
          </a-button>
        </a-space>
      </div>
    </a-card>
  </div>
</template>

<script>
import axios from 'axios';
import VirtualKeyboard from './VirtualKeyboard.vue';

export default {
  name: 'EnglishTypingPractice',
  components: {
    VirtualKeyboard
  },
  data() {
    return {
      englishTexts: [
        { id: 1, title: '基础练习', content: 'the quick brown fox jumps over the lazy dog', difficulty: '简单' },
        { id: 2, title: '常用句子', content: 'hello world this is a typing practice text for english learning', difficulty: '简单' },
        { id: 3, title: '进阶练习', content: 'practice makes perfect keep typing to improve your speed and accuracy', difficulty: '中等' },
      ],
      currentText: null,
      selectedTextId: null,
      userInput: '',
      startTime: null,
      elapsedTime: 0,
      timer: null,
      currentCharIndex: 0,
      practiceMode: 'manual',
      typingSpeed: 100,
      isAutoTyping: false,
      isPaused: false,
      autoTimer: null
    };
  },
  computed: {
    accuracy() {
      if (!this.currentText || this.currentText.content.length === 0) return 0;
      const originalText = this.currentText.content;
      const typedText = this.userInput;
      
      let correctChars = 0;
      for (let i = 0; i < Math.min(originalText.length, typedText.length); i++) {
        if (originalText[i] === typedText[i]) {
          correctChars++;
        }
      }
      
      return originalText.length > 0 ? ((correctChars / originalText.length) * 100).toFixed(2) : 0;
    },
    speed() {
      if (!this.startTime || this.elapsedTime === 0 || !this.currentText) return 0;
      const minutes = this.elapsedTime / 60000;
      const wordsTyped = this.userInput.split(/\s+/).filter(w => w.length > 0).length;
      return (wordsTyped / minutes).toFixed(2);
    },
    currentChar() {
      if (!this.currentText || this.currentText.content.length === 0) return null;
      if (this.currentCharIndex >= this.currentText.content.length) return null;
      return this.currentText.content[this.currentCharIndex];
    },
    currentActiveKey() {
      if (!this.currentText || this.currentCharIndex >= this.currentText.content.length) return null;
      const char = this.currentText.content[this.currentCharIndex];
      return char.toLowerCase();
    }
  },
  watch: {
    practiceMode() {
      this.resetPractice();
    }
  },
  async mounted() {
    await this.loadData();
    window.addEventListener('keydown', this.handleGlobalKeyDown);
  },
  beforeUnmount() {
    window.removeEventListener('keydown', this.handleGlobalKeyDown);
    if (this.timer) {
      clearInterval(this.timer);
    }
    if (this.autoTimer) {
      clearTimeout(this.autoTimer);
    }
  },
  methods: {
    async loadData() {
      try {
        const res = await axios.get('/api/english-texts').catch(() => ({ data: [] }));
        if (res.data && res.data.length > 0) {
          this.englishTexts = res.data;
        }
        if (this.englishTexts.length > 0) {
          this.selectedTextId = this.englishTexts[0].id;
          this.currentText = this.englishTexts[0];
        }
      } catch (error) {
        console.error('加载英语文章失败:', error);
      }
    },
    handleTextChange(value) {
      this.selectedTextId = value;
      this.currentText = this.englishTexts.find(t => t.id === value) || null;
      this.resetPractice();
    },
    startTypingPractice() {
      if (!this.startTime) {
        this.startTime = Date.now();
        this.timer = setInterval(() => {
          this.elapsedTime = Date.now() - this.startTime;
        }, 1000);
      }
    },
    resetPractice() {
      this.userInput = '';
      this.currentCharIndex = 0;
      if (this.timer) {
        clearInterval(this.timer);
        this.timer = null;
      }
      if (this.autoTimer) {
        clearTimeout(this.autoTimer);
        this.autoTimer = null;
      }
      this.startTime = null;
      this.elapsedTime = 0;
      this.isAutoTyping = false;
      this.isPaused = false;
    },
    getCharClass(index) {
      const classes = ['char'];
      if (index < this.userInput.length) {
        classes.push(this.userInput[index] === this.currentText.content[index] ? 'correct' : 'incorrect');
      } else if (index === this.userInput.length) {
        classes.push('current');
      }
      return classes.join(' ');
    },
    handleUserInput() {
      this.currentCharIndex = this.userInput.length;
    },
    startAutoTyping() {
      if (!this.currentText || !this.currentText.content) return;
      this.isAutoTyping = true;
      this.isPaused = false;
      this.startTime = Date.now();
      this.timer = setInterval(() => {
        this.elapsedTime = Date.now() - this.startTime;
      }, 1000);
      this.typeNextChar();
    },
    typeNextChar() {
      if (!this.currentText || this.isPaused) return;
      const content = this.currentText.content;
      if (this.userInput.length >= content.length) {
        this.isAutoTyping = false;
        if (this.timer) {
          clearInterval(this.timer);
          this.timer = null;
        }
        return;
      }
      const nextChar = content[this.userInput.length];
      this.userInput += nextChar;
      this.currentCharIndex = this.userInput.length;
      this.handleUserInput();
      this.autoTimer = setTimeout(() => {
        this.typeNextChar();
      }, this.typingSpeed);
    },
    pauseAutoTyping() {
      this.isPaused = true;
      if (this.autoTimer) {
        clearTimeout(this.autoTimer);
        this.autoTimer = null;
      }
    },
    resumeAutoTyping() {
      this.isPaused = false;
      this.typeNextChar();
    },
    formatTime(ms) {
      const seconds = Math.floor(ms / 1000);
      const mins = Math.floor(seconds / 60);
      const secs = seconds % 60;
      return `${mins.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`;
    },
    handleGlobalKeyDown(event) {
      if (this.practiceMode === 'manual' && this.currentText) {
        const char = event.key;
        if (char === 'Backspace') {
          event.preventDefault();
          if (this.userInput.length > 0) {
            this.userInput = this.userInput.slice(0, -1);
            this.currentCharIndex = this.userInput.length;
          }
          return;
        }
        if (char.length === 1) {
          event.preventDefault();
          this.userInput += char;
          this.currentCharIndex = this.userInput.length;
          this.handleUserInput();
          if (!this.startTime) {
            this.startTypingPractice();
          }
        }
      }
    }
  },
};
</script>

<style scoped>
.english-practice-container {
  max-width: 1000px;
  margin: 0 auto;
  padding: 20px;
}

.practice-card {
  width: 100%;
}

.stats-container {
  margin-bottom: 20px;
}

.mode-control {
  margin-bottom: 16px;
  padding: 12px;
  background-color: #f0f5ff;
  border-radius: 4px;
}

.auto-display {
  margin-top: 16px;
}

.original-text {
  padding: 16px;
  background-color: #f9f9f9;
  border: 1px solid #ddd;
  border-radius: 4px;
  min-height: 100px;
  max-height: 200px;
  overflow-y: auto;
  line-height: 1.8;
  font-size: 16px;
}

.char {
  padding: 2px;
}

.char.correct {
  background-color: #bbf7d0;
  color: #166534;
}

.char.incorrect {
  background-color: #fecaca;
  color: #b91c1c;
  text-decoration: underline;
}

.char.current {
  background-color: #dbeafe;
  color: #1e40af;
  animation: blink 1s infinite;
}

@keyframes blink {
  0%, 50% { opacity: 1; }
  51%, 100% { opacity: 0.5; }
}

.input-section {
  margin-top: 16px;
}

.control-buttons {
  margin-top: 16px;
  text-align: center;
}

.current-char-hint {
  margin-top: 16px;
}
</style>