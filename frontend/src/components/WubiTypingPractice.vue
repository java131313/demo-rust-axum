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
          >
            {{ char }}
          </span>
        </div>
      </div>
      
      <!-- 输入区域 -->
      <div class="input-section">
        <a-textarea
          v-model:value="userInput"
          placeholder="请在此输入文字进行练习..."
          :auto-size="{ minRows: 4, maxRows: 6 }"
          @focus="startTypingPractice"
          style="margin-top: 16px;"
        />
      </div>
      
      <!-- 控制按钮 -->
      <div class="control-buttons">
        <a-button @click="resetPractice" type="primary" danger>
          重新开始
        </a-button>
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
      
      <div v-if="searchResult" class="search-result">
        <a-alert
          message="查询结果"
          type="info"
          :description="`${searchResult.character}: ${searchResult.code} (${searchResult.position}) - ${searchResult.description}`"
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

export default {
  name: 'WubiTypingPractice',
  components: {
    WubiGraph
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
      searchCharacter: '',
      searchResult: null,
      wubiRoots: [] // 添加wubiRoots数据
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
      const minutes = this.elapsedTime / 60000; // Convert ms to minutes
      const charsTyped = this.userInput.length;
      return (charsTyped / minutes).toFixed(2);
    }
  },
  async mounted() {
    await this.loadData();
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
        
        this.articles = articlesRes.data;
        this.wubiRoots = wubiRootsRes.data;
        
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
      if (this.timer) {
        clearInterval(this.timer);
        this.timer = null;
      }
      this.startTime = null;
      this.elapsedTime = 0;
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
      if (!value.trim()) {
        this.searchResult = null;
        return;
      }
      
      try {
        const response = await axios.get(`/api/search-wubi-root/${encodeURIComponent(value)}`);
        this.searchResult = response.data;
      } catch (error) {
        console.error('搜索字根失败:', error);
        this.searchResult = null;
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
    }
  },
  beforeUnmount() {
    if (this.timer) {
      clearInterval(this.timer);
    }
  }
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