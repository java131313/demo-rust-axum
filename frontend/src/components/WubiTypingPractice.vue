<template>
  <div class="wubi-practice-container">
    <a-card class="practice-card">
      <template #title>
        <span>五笔打字练习</span>
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
            :title="`点击查看${char}的五笔编码`"
            style="cursor: pointer;"
          >
            {{ char }}
          </span>
        </div>
      </div>
      
      <!-- 五笔编码提示 -->
      <div class="wubi-hint-section" v-if="currentCharacter || wubiCodeError || isLoadingWubiCode">
        <a-alert
          v-if="currentWubiCode"
          :message="`当前字符: ${currentCharacter}`"
          :description="`五笔编码: ${currentWubiCode.full_code}`"
          type="info"
          show-icon
          style="margin-bottom: 16px;"
        />
        <a-alert
          v-else-if="isLoadingWubiCode"
          message="正在查询五笔编码..."
          type="info"
          show-icon
          style="margin-bottom: 16px;"
        />
        <a-alert
          v-else-if="wubiCodeError"
          :message="wubiCodeError"
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
      
      <!-- 虚拟键盘指法提示 -->
      <VirtualKeyboard :activeKey="currentActiveKey" :wubiCode="currentWubiCode?.full_code" :codeIndex="wubiCodeIndex" />
      
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
    
    <!-- 字根表卡片 -->
    <a-card class="wubi-roots-card" title="五笔字根表">
      <div class="wubi-search">
        <a-input-search
          v-model:value="searchCharacter"
          placeholder="输入汉字查询五笔编码"
          enter-button
          @search="searchWubiRoot"
        />
      </div>
      
      <div v-if="searchError" class="search-result">
        <a-alert message="无法查询" type="warning" :description="searchError" show-icon />
      </div>
      <div v-else-if="searchResult" class="search-result">
        <a-alert
          message="查询结果"
          type="info"
          :description="wubiSearchResultDescription"
          show-icon
        />
      </div>
      
      <!-- 使用G6展示字根关系图 -->
      <WubiGraph :wubiRoots="wubiRoots" />
    </a-card>
  </div>
</template>

<script>
import axios from 'axios';
import WubiGraph from './WubiGraph.vue';
import VirtualKeyboard from './VirtualKeyboard.vue';

export default {
  name: 'WubiTypingPractice',
  components: {
    WubiGraph,
    VirtualKeyboard
  },
  data() {
    return {
      articles: [],
      currentArticle: null,
      selectedArticleId: null,
      userInput: '',
      wubiInput: '', // 用于存储五笔编码输入
      startTime: null,
      elapsedTime: 0,
      timer: null,
      searchCharacter: '',
      searchResult: null,
      searchError: null,
      wubiRoots: [],
      currentCharIndex: 0,
      currentWubiCode: null,
      wubiCodeCache: {},
      isLoadingWubiCode: false,
      wubiCodeError: null,
      wubiLetterCount: 0,
      lastCharStartPosition: 0,
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
    wubiCodeIndex() {
      return this.wubiLetterCount;
    },
    currentActiveKey() {
      if (this.currentWubiCode && this.currentWubiCode.full_code && this.wubiLetterCount < this.currentWubiCode.full_code.length) {
        return this.currentWubiCode.full_code[this.wubiLetterCount];
      }
      
      if (!this.currentArticle || !this.currentArticle.content.length) return null;
      if (this.currentCharIndex >= this.currentArticle.content.length) return null;
      
      const char = this.currentArticle.content[this.currentCharIndex];
      if (/[\u4e00-\u9fa5]/.test(char)) {
        return null;
      }
      
      return char.toLowerCase();
    },
    wubiSearchResultDescription() {
      if (!this.searchResult) return '';
      const r = this.searchResult;
      const parts = [
        `汉字：${r.character}`,
        `全码：${r.full_code || '—'}`,
        `简码：${r.simple_code || '—'}`,
      ];
      if (r.pinyin) parts.push(`拼音：${r.pinyin}`);
      if (r.remark) parts.push(`备注：${r.remark}`);
      return parts.join('；');
    }
  },
  watch: {
    practiceMode() {
      this.resetPractice();
    },
    searchCharacter(val) {
      if (!val || !String(val).trim()) {
        this.searchResult = null;
        this.searchError = null;
      }
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
        // 并行加载数据，但分别处理错误
        const [articlesRes, wubiRootsRes] = await Promise.all([
          axios.get('/api/articles').catch(error => {
            console.error('加载文章失败:', error);
            return { data: [] }; // 返回空数组作为默认值
          }),
          axios.get('/api/wubi-roots').catch(error => {
            console.error('加载字根失败:', error);
            return { data: [] }; // 返回空数组作为默认值
          })
        ]);
        
        console.log('API响应 - 文章:', articlesRes.data);
        console.log('API响应 - 字根:', wubiRootsRes.data);
        
        // axios 响应拦截器可能修改了data，确保获取正确的数据
        const articlesData = articlesRes.data || articlesRes;
        const wubiRootsData = wubiRootsRes.data || wubiRootsRes;
        
        // 如果返回的数据在data属性中（axios标准格式）
        this.articles = Array.isArray(articlesData) ? articlesData : (articlesData.data || []);
        this.wubiRoots = Array.isArray(wubiRootsData) ? wubiRootsData : (wubiRootsData.data || []);
        
        console.log('处理后的文章数据:', this.articles);
        
        if (this.articles.length > 0) {
          this.selectedArticleId = this.articles[0].id;
          this.currentArticle = this.articles[0];
        }
      } catch (error) {
        console.error('加载数据失败:', error);
      }
    },
    handleArticleChange(value) {
      this.selectedArticleId = value;
      this.currentArticle = this.articles.find(a => a.id === value) || null;
      this.resetPractice();
      // 清空缓存，因为不同文章可能有不同的字符
      this.wubiCodeCache = {};
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
      this.wubiInput = '';
      this.currentCharIndex = 0;
      this.currentWubiCode = null;
      this.wubiCodeError = null;
      this.wubiLetterCount = 0;
      this.lastCharStartPosition = 0;
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
    async searchWubiRoot(value) {
      const raw = typeof value === 'string' ? value : this.searchCharacter;
      const trimmed = raw.trim();
      this.searchError = null;
      if (!trimmed) {
        this.searchResult = null;
        return;
      }

      const chars = [...trimmed];
      if (chars.length !== 1) {
        this.searchResult = null;
        this.searchError = '请只输入一个汉字再查询。';
        return;
      }
      const char = chars[0];
      if (!/[\u4e00-\u9fff]/.test(char)) {
        this.searchResult = null;
        this.searchError = '请输入汉字。';
        return;
      }

      try {
        const response = await axios.get(`/api/wubi/${encodeURIComponent(char)}`);
        this.searchResult = response.data;
      } catch (error) {
        console.error('查询五笔编码失败:', error);
        this.searchResult = null;
        if (error.response?.status === 404) {
          this.searchError = `词库中未找到「${char}」的五笔编码。`;
        } else if (error.response?.status === 400) {
          this.searchError = '请求无效，请确认输入的是单个汉字。';
        } else {
          this.searchError = error.message || '查询失败，请稍后重试。';
        }
      }
    },
    handleUserInput() {
      this.currentCharIndex = this.userInput.length;

      if (this.currentCharacter) {
        const typedSinceCurrent = this.wubiInput.length;
        if (this.currentWubiCode && this.currentWubiCode.full_code) {
          const codeLen = this.currentWubiCode.full_code.length;
          this.wubiLetterCount = Math.min(typedSinceCurrent, codeLen);
          if (typedSinceCurrent >= codeLen) {
            // 编码输入完成，检查是否正确
            if (this.wubiInput === this.currentWubiCode.full_code) {
              // 编码正确，添加汉字到输入框
              this.userInput += this.currentCharacter;
              this.wubiInput = '';
              this.lastCharStartPosition = this.userInput.length;
              this.wubiLetterCount = 0;
            } else {
              // 编码错误，重置输入
              this.wubiInput = '';
              this.wubiLetterCount = 0;
            }
          }
        } else {
          this.wubiLetterCount = 0;
        }
        this.fetchWubiCodeForCurrentChar();
      } else {
        this.currentWubiCode = null;
        this.wubiCodeError = null;
        this.isLoadingWubiCode = false;
        this.wubiLetterCount = 0;
        this.wubiInput = '';

        if (this.currentCharIndex < this.currentArticle.content.length) {
          const char = this.currentArticle.content[this.currentCharIndex];
          if (char.trim() && !/[\u4e00-\u9fa5]/.test(char)) {
            this.wubiCodeError = `"${char}"不是汉字，无需五笔编码`;
          }
        }
      }
    },
    async fetchWubiCodeForCurrentChar() {
      const char = this.currentCharacter;

      if (this.wubiCodeCache[char]) {
        this.currentWubiCode = this.wubiCodeCache[char];
        this.wubiCodeError = null;
        const codeLen = this.currentWubiCode.full_code.length;
        const typedSinceCurrent = this.wubiInput.length;
        this.wubiLetterCount = Math.min(typedSinceCurrent, codeLen);
        if (typedSinceCurrent >= codeLen) {
          this.lastCharStartPosition = this.userInput.length;
          this.wubiLetterCount = 0;
        }
        return;
      }

      this.currentWubiCode = null;
      this.wubiCodeError = null;
      this.isLoadingWubiCode = true;
      this.wubiLetterCount = 0;

      try {
        const response = await axios.get(`/api/wubi/${encodeURIComponent(char)}`);
        const wubiData = response.data;

        this.wubiCodeCache[char] = wubiData;
        this.currentWubiCode = wubiData;
        this.wubiCodeError = null;
      } catch (error) {
        console.error(`获取"${char}"的五笔编码失败:`, error);

        if (error.response && error.response.status === 404) {
          this.wubiCodeError = `未找到"${char}"的五笔编码`;
        } else if (error.response && error.response.status === 400) {
          this.wubiCodeError = `"${char}"不是有效的单个汉字`;
        } else {
          this.wubiCodeError = `查询失败: ${error.message}`;
        }

        this.currentWubiCode = null;
      } finally {
        this.isLoadingWubiCode = false;
      }
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
      
      // 更新五笔编码提示
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
      // 如果点击的位置在当前输入长度之前，直接显示该字符的五笔编码
      if (index < this.currentArticle.content.length) {
        const char = this.currentArticle.content[index];

        // 检查是否是汉字
        if (/[\u4e00-\u9fa5]/.test(char)) {
          // 如果缓存中有，直接显示
          if (this.wubiCodeCache[char]) {
            this.currentWubiCode = this.wubiCodeCache[char];
            this.wubiCodeError = null;
          } else {
            // 否则获取编码
            this.fetchWubiCodeForChar(char);
          }
        } else {
          this.currentWubiCode = null;
          this.wubiCodeError = `"${char}"不是汉字，无需五笔编码`;
        }
      }
    },
    async fetchWubiCodeForChar(char) {
      // 检查缓存
      if (this.wubiCodeCache[char]) {
        this.currentWubiCode = this.wubiCodeCache[char];
        this.wubiCodeError = null;
        return;
      }

      // 重置状态
      this.currentWubiCode = null;
      this.wubiCodeError = null;
      this.isLoadingWubiCode = true;

      try {
        const response = await axios.get(`/api/wubi/${encodeURIComponent(char)}`);
        const wubiData = response.data;

        // 缓存结果
        this.wubiCodeCache[char] = wubiData;
        this.currentWubiCode = wubiData;
        this.wubiCodeError = null;
      } catch (error) {
        console.error(`获取"${char}"的五笔编码失败:`, error);

        // 处理不同的错误情况
        if (error.response && error.response.status === 404) {
          this.wubiCodeError = `未找到"${char}"的五笔编码`;
        } else if (error.response && error.response.status === 400) {
          this.wubiCodeError = `"${char}"不是有效的单个汉字`;
        } else {
          this.wubiCodeError = `查询失败: ${error.message}`;
        }

        this.currentWubiCode = null;
      } finally {
        this.isLoadingWubiCode = false;
      }
    },
    formatTime(ms) {
      const seconds = Math.floor(ms / 1000);
      const mins = Math.floor(seconds / 60);
      const secs = seconds % 60;
      return `${mins.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`;
    },
    initWubiGraph() {
      // 这里初始化G6图表，展示字根关系
      // 由于G6需要复杂的配置，这里仅做初始化
      console.log('初始化G6字根关系图');
    },
    handleGlobalKeyDown(event) {
      const t = event.target;
      if (
        t instanceof HTMLElement &&
        (t.tagName === 'INPUT' ||
          t.tagName === 'TEXTAREA' ||
          t.tagName === 'SELECT' ||
          t.isContentEditable)
      ) {
        return;
      }
      if (this.practiceMode === 'manual' && this.currentArticle) {
        const char = event.key;
        if (char === 'Backspace') {
          event.preventDefault();
          if (this.wubiInput.length > 0) {
            this.wubiInput = this.wubiInput.slice(0, -1);
            this.handleUserInput();
          } else if (this.userInput.length > 0) {
            this.userInput = this.userInput.slice(0, -1);
            this.currentCharIndex = this.userInput.length;
            this.handleUserInput();
          }
          return;
        }
        if (char.length === 1 || char === ' ') {
          event.preventDefault();
          if (this.currentCharacter) {
            // 对于汉字，输入到wubiInput
            if (/[a-zA-Z]/.test(char)) {
              this.wubiInput += char.toLowerCase();
            }
          } else {
            // 对于非汉字，直接输入到userInput
            this.userInput += char === ' ' ? ' ' : char;
          }
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
.wubi-practice-container {
  display: grid;
  grid-template-columns: 1fr 400px;
  gap: 20px;
}

.practice-card {
  width: 100%;
}

.wubi-roots-card {
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

.wubi-hint-section {
  margin-top: 16px;
  animation: fadeIn 0.3s ease-in;
}

@keyframes fadeIn {
  from { opacity: 0; transform: translateY(-10px); }
  to { opacity: 1; transform: translateY(0); }
}

.wubi-search {
  margin-bottom: 16px;
}

.search-result {
  margin-bottom: 16px;
}

.wubi-graph {
  width: 100%;
  height: 400px;
  border: 1px solid #ddd;
  border-radius: 4px;
}

@media (max-width: 768px) {
  .wubi-practice-container {
    grid-template-columns: 1fr;
  }
}
</style>