<template>
  <div class="pinyin-practice-container">
    <a-card class="practice-card">
      <template #title>
        <span>拼音打字练习</span>
      </template>
      
      <!-- 练习统计 -->
      <div class="stats-container">
        <a-row :gutter="16">
          <a-col :span="8">
            <a-statistic title="时间" :value="formatTime(elapsedTime)" />
          </a-col>
          <a-col :span="8">
            <a-statistic title="准确率" :value="accuracy" suffix="%" />
          </a-col>
          <a-col :span="8">
            <a-statistic title="速度" :value="speed" suffix="字/分钟" />
          </a-col>
        </a-row>
      </div>
      
      <!-- 模式选择和速度控制 -->
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
            <a-select-option :value="500">慢速</a-select-option>
            <a-select-option :value="250">正常</a-select-option>
            <a-select-option :value="100">快速</a-select-option>
          </a-select>
        </a-space>
      </div>

      <!-- 文章选择 -->
      <div class="article-selection">
        <a-select 
          v-model:value="selectedArticleId" 
          placeholder="选择练习文章"
          style="width: 100%; margin-bottom: 16px;"
          @change="handleArticleChange"
        >
          <a-select-option 
            v-for="article in articles" 
            :key="article.id" 
            :value="article.id"
          >
            {{ article.title }} ({{ article.difficulty }})
          </a-select-option>
        </a-select>
      </div>
      
      <!-- 原文显示 -->
      <div class="original-text" v-if="currentArticle">
        <div class="text-content">
          <span
            v-for="(char, index) in currentArticle.content.split('')"
            :key="index"
            :class="getCharClass(index)"
            @click="handleCharClick(index)"
            :title="`点击查看${char}的拼音`"
            style="cursor: pointer;"
          >
            {{ char }}
          </span>
        </div>
      </div>
      
      <!-- 拼音提示 -->
      <div class="pinyin-hint-section" v-if="currentCharacter || pinyinError || isLoadingPinyin">
        <a-alert
          v-if="currentPinyin"
          :message="`当前字符: ${currentCharacter}`"
          :description="`拼音: ${currentPinyin}`"
          type="info"
          show-icon
          style="margin-bottom: 16px;"
        />
        <a-alert
          v-else-if="isLoadingPinyin"
          message="正在查询拼音..."
          type="info"
          show-icon
          style="margin-bottom: 16px;"
        />
        <a-alert
          v-else-if="pinyinError"
          :message="pinyinError"
          type="warning"
          show-icon
          style="margin-bottom: 16px;"
        />
      </div>

      <!-- 输入区域 -->
      <div class="input-section">
        <!-- 自动演示显示框 -->
        <div v-if="practiceMode === 'auto'" class="auto-display">
          <a-textarea
            v-model:value="userInput"
            placeholder="自动演示中..."
            :auto-size="{ minRows: 4, maxRows: 6 }"
            disabled
            style="margin-bottom: 16px;"
          />
        </div>
        
        <!-- 手动输入框 -->
        <a-textarea
          v-model:value="userInput"
          placeholder="请在此输入文字进行练习..."
          :auto-size="{ minRows: 4, maxRows: 6 }"
          @focus="startTypingPractice"
          @input="handleUserInput"
          style="margin-top: 16px;"
        />
      </div>
      
      <!-- 控制按钮 -->
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

export default {
  name: 'PinyinTypingPractice',
  inject: {
    virtualKeyboard: {
      default() {
        return {
          setKeyboard() {},
          resetKeyboard() {},
          registerKeyboardSync() {
            return () => {};
          },
        };
      },
    },
    homeTabs: {
      default() {
        return { activeTabKey: { value: '4' } };
      },
    },
  },
  data() {
    return {
      articles: [],
      currentArticle: null,
      selectedArticleId: null,
      userInput: '',
      startTime: null,
      elapsedTime: 0,
      timer: null,
      currentCharIndex: 0,
      currentPinyin: null,
      pinyinCache: {},
      isLoadingPinyin: false,
      pinyinError: null,
      // 自动打字相关
      practiceMode: 'manual', // 'manual' 或 'auto'
      typingSpeed: 250, // 毫秒
      isAutoTyping: false,
      isPaused: false,
      autoTimer: null
    };
  },
  computed: {
    accuracy() {
      if (!this.currentArticle || this.currentArticle.content.length === 0) return 0;
      const originalText = this.currentArticle.content.replace(/\s+/g, '');
      const typedText = this.userInput.replace(/\s+/g, '');
      
      let correctChars = 0;
      for (let i = 0; i < Math.min(originalText.length, typedText.length); i++) {
        if (originalText[i] === typedText[i]) {
          correctChars++;
        }
      }
      
      return originalText.length > 0 ? ((correctChars / originalText.length) * 100).toFixed(2) : 0;
    },
    speed() {
      if (!this.startTime || this.elapsedTime === 0 || !this.currentArticle) return 0;
      const minutes = this.elapsedTime / 60000;
      const charsTyped = this.userInput.length;
      return (charsTyped / minutes).toFixed(2);
    },
    currentCharacter() {
      if (!this.currentArticle || this.currentArticle.content.length === 0) return null;
      if (this.currentCharIndex >= this.currentArticle.content.length) return null;

      const char = this.currentArticle.content[this.currentCharIndex];
      const isChineseChar = /[\u4e00-\u9fa5]/.test(char);
      return isChineseChar ? char : null;
    },
    pinyinKeyboardHint() {
      const py = this.currentPinyin;
      if (!py || py === '未知') return null;
      const m = String(py).match(/[a-zA-Z]/);
      return m ? m[0].toLowerCase() : null;
    },
  },
  watch: {
    currentPinyin() {
      this.syncKeyboardToHost();
    },
    currentCharacter() {
      this.syncKeyboardToHost();
    },
    currentCharIndex() {
      this.syncKeyboardToHost();
    },
  },
  async mounted() {
    await this.loadData();
    this._vkUnreg = this.virtualKeyboard.registerKeyboardSync('4', this.syncKeyboardToHost);
    this.syncKeyboardToHost();
  },
  beforeUnmount() {
    if (this._vkUnreg) this._vkUnreg();
    if (this.timer) {
      clearInterval(this.timer);
    }
    if (this.autoTimer) {
      clearTimeout(this.autoTimer);
    }
  },
  methods: {
    activeHomeTabKey() {
      const tab = this.homeTabs?.activeTabKey;
      if (tab && typeof tab === 'object' && 'value' in tab) return tab.value;
      return tab;
    },
    syncKeyboardToHost() {
      if (this.activeHomeTabKey() !== '4') return;
      this.virtualKeyboard.setKeyboard({
        activeKey: this.pinyinKeyboardHint,
        wubiCode: null,
        codeIndex: 0,
      });
    },
    async loadData() {
      try {
        const response = await axios.get('/api/articles');
        this.articles = response.data || [];
        
        if (this.articles.length > 0) {
          this.selectedArticleId = this.articles[0].id;
          this.currentArticle = this.articles[0];
        }
      } catch (error) {
        console.error('加载文章失败:', error);
      }
    },
    handleArticleChange(value) {
      this.selectedArticleId = value;
      this.currentArticle = this.articles.find(a => a.id === value) || null;
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
      this.currentPinyin = null;
      this.pinyinError = null;
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
        classes.push(this.userInput[index] === this.currentArticle.content[index] ? 'correct' : 'incorrect');
      } else if (index === this.userInput.length) {
        classes.push('current');
      }
      return classes.join(' ');
    },
    handleUserInput() {
      this.currentCharIndex = this.userInput.length;

      if (this.currentCharacter) {
        this.fetchPinyinForCurrentChar();
      } else {
        this.currentPinyin = null;
        this.pinyinError = null;
        this.isLoadingPinyin = false;

        if (this.currentCharIndex < this.currentArticle.content.length) {
          const char = this.currentArticle.content[this.currentCharIndex];
          if (char.trim() && !/[\u4e00-\u9fa5]/.test(char)) {
            this.pinyinError = `"${char}"不是汉字，无需拼音`;
          }
        }
      }
    },
    async fetchPinyinForCurrentChar() {
      const char = this.currentCharacter;

      if (this.pinyinCache[char]) {
        this.currentPinyin = this.pinyinCache[char];
        this.pinyinError = null;
        return;
      }

      this.currentPinyin = null;
      this.pinyinError = null;
      this.isLoadingPinyin = true;

      try {
        const response = await axios.get(`/api/wubi/${encodeURIComponent(char)}`);
        const wubiData = response.data;
        const pinyin = wubiData.pinyin || wubiData.simple_pinyin;

        this.pinyinCache[char] = pinyin || '未知';
        this.currentPinyin = pinyin || '未知';
        this.pinyinError = null;
      } catch (error) {
        console.error(`获取"${char}"的拼音失败:`, error);

        this.pinyinCache[char] = this.getMockPinyin(char);
        this.currentPinyin = this.pinyinCache[char];
        this.pinyinError = null;
      } finally {
        this.isLoadingPinyin = false;
      }
    },
    getMockPinyin(char) {
      const pinyinMap = {
        '你': 'ni',
        '好': 'hao',
        '我': 'wo',
        '他': 'ta',
        '她': 'ta',
        '它': 'ta',
        '人': 'ren',
        '口': 'kou',
        '日': 'ri',
        '月': 'yue',
        '水': 'shui',
        '火': 'huo',
        '土': 'tu',
        '金': 'jin',
        '木': 'mu',
        '刀': 'dao',
        '力': 'li',
        '大': 'da',
        '小': 'xiao'
      };
      return pinyinMap[char] || '未知';
    },
    startAutoTyping() {
      if (!this.currentArticle || !this.currentArticle.content) return;
      
      this.isAutoTyping = true;
      this.isPaused = false;
      this.startTime = Date.now();
      this.timer = setInterval(() => {
        this.elapsedTime = Date.now() - this.startTime;
      }, 1000);
      
      this.typeNextChar();
    },
    typeNextChar() {
      if (!this.currentArticle || this.isPaused) return;
      
      const content = this.currentArticle.content;
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
      
      // 更新拼音提示
      this.handleUserInput();
      
      // 根据速度设置，延迟输入下一个字符
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
    handleCharClick(index) {
      // 如果点击的位置在当前输入长度之前，直接显示该字符的拼音
      if (index < this.currentArticle.content.length) {
        const char = this.currentArticle.content[index];

        // 检查是否是汉字
        if (/[\u4e00-\u9fa5]/.test(char)) {
          // 如果缓存中有，直接显示
          if (this.pinyinCache[char]) {
            this.currentPinyin = this.pinyinCache[char];
            this.pinyinError = null;
          } else {
            // 否则获取拼音
            this.fetchPinyinForChar(char);
          }
        } else {
          this.currentPinyin = null;
          this.pinyinError = `"${char}"不是汉字，无需拼音`;
        }
      }
    },
    async fetchPinyinForChar(char) {
      // 检查缓存
      if (this.pinyinCache[char]) {
        this.currentPinyin = this.pinyinCache[char];
        this.pinyinError = null;
        return;
      }

      // 重置状态
      this.currentPinyin = null;
      this.pinyinError = null;
      this.isLoadingPinyin = true;

      try {
        const response = await axios.get(`/api/wubi/${encodeURIComponent(char)}`);
        const wubiData = response.data;
        const pinyin = wubiData.pinyin || wubiData.simple_pinyin;

        this.pinyinCache[char] = pinyin || '未知';
        this.currentPinyin = pinyin || '未知';
        this.pinyinError = null;
      } catch (error) {
        console.error(`获取"${char}"的拼音失败:`, error);

        this.pinyinCache[char] = this.getMockPinyin(char);
        this.currentPinyin = this.pinyinCache[char];
        this.pinyinError = null;
      } finally {
        this.isLoadingPinyin = false;
      }
    },
    formatTime(ms) {
      const seconds = Math.floor(ms / 1000);
      const mins = Math.floor(seconds / 60);
      const secs = seconds % 60;
      return `${mins.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`;
    }
  },
};
</script>

<style scoped>
.pinyin-practice-container {
  width: 100%;
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

.pinyin-hint-section {
  margin-top: 16px;
  animation: fadeIn 0.3s ease-in;
}

@keyframes fadeIn {
  from { opacity: 0; transform: translateY(-10px); }
  to { opacity: 1; transform: translateY(0); }
}
</style>