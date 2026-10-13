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
        <a-space style="margin-bottom: 16px;">
          <a-button @click="openAddCustomArticleModal" type="dashed">
            添加自定义文章
          </a-button>
          <template v-if="currentText && currentText.isCustom">
            <a-button @click="openEditCustomArticleModal" type="dashed">编辑</a-button>
            <a-button @click="handleDeleteCustomArticle" type="dashed" danger>删除</a-button>
          </template>
        </a-space>
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
          :description="String(currentChar)"
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
            <!-- 添加/编辑自定义文章弹窗 -->
      <a-modal
        v-model:open="showCustomArticleModal"
        :title="editingCustomArticleId === null ? '添加自定义英语文章' : '编辑自定义文章'"
        @ok="handleSaveCustomArticle"
        @cancel="closeCustomArticleModal"
        :confirm-loading="isSavingCustomArticle"
      >
        <a-form layout="vertical">
          <a-form-item label="文章标题" required>
            <a-input v-model:value="customArticleTitle" placeholder="请输入文章标题" maxlength="50" />
          </a-form-item>
          <a-form-item label="文章内容" required>
            <a-textarea
              v-model:value="customArticleContent"
              placeholder="请输入英语文章内容"
              :auto-size="{ minRows: 4, maxRows: 8 }"
              maxlength="5000"
              show-count
            />
          </a-form-item>
        </a-form>
      </a-modal>
    </a-card>
  </div>
</template>

<script>
import axios from 'axios';
import { Modal } from 'ant-design-vue';

export default {
  name: 'EnglishTypingPractice',
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
        return { activeTabKey: { value: '3' } };
      },
    },
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
      // 自定义文章相关
      showCustomArticleModal: false,
      customArticleTitle: '',
      customArticleContent: '',
      isSavingCustomArticle: false,
      editingCustomArticleId: null,
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
    },
    currentActiveKey() {
      this.syncKeyboardToHost();
    },
    userInput() {
      this.syncKeyboardToHost();
    },
  },
  async mounted() {
    await this.loadData();
    this._vkUnreg = this.virtualKeyboard.registerKeyboardSync('3', this.syncKeyboardToHost);
    this.syncKeyboardToHost();
    window.addEventListener('keydown', this.handleGlobalKeyDown);
  },
  beforeUnmount() {
    if (this._vkUnreg) this._vkUnreg();
    window.removeEventListener('keydown', this.handleGlobalKeyDown);
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
      if (this.activeHomeTabKey() !== '3') return;
      this.virtualKeyboard.setKeyboard({
        activeKey: this.currentActiveKey,
        wubiCode: null,
        codeIndex: 0,
      });
    },
    async loadData() {
      try {
        const [res, customRes] = await Promise.all([
          axios.get('/api/english-texts').catch(() => ({ data: [] })),
          axios.get('/api/custom-articles', { params: { lang: 'en' } }).catch(() => ({ data: [] })),
        ]);
        if (res.data && res.data.length > 0) {
          this.englishTexts = res.data.map(t => ({ ...t, isCustom: false }));
        }
        // 自定义英语文章：id加custom-前缀避免与系统文章冲突
        const customTexts = (customRes.data || []).map(a => ({
          ...a,
          id: `custom-${a.id}`,
          realId: a.id,
          isCustom: true,
          difficulty: a.difficulty === 'custom' ? '自定义' : a.difficulty,
        }));
        this.englishTexts = [...this.englishTexts.filter(t => !t.isCustom), ...customTexts];
        if (this.englishTexts.length > 0) {
          this.selectedTextId = this.englishTexts[0].id;
          this.currentText = this.englishTexts[0];
        }
      } catch (error) {
        console.error('加载英语文章失败:', error);
      }
    },
    openAddCustomArticleModal() {
      this.editingCustomArticleId = null;
      this.customArticleTitle = '';
      this.customArticleContent = '';
      this.showCustomArticleModal = true;
    },
    openEditCustomArticleModal() {
      if (!this.currentText || !this.currentText.isCustom) return;
      this.editingCustomArticleId = this.currentText.realId;
      this.customArticleTitle = this.currentText.title;
      this.customArticleContent = this.currentText.content;
      this.showCustomArticleModal = true;
    },
    closeCustomArticleModal() {
      this.showCustomArticleModal = false;
      this.customArticleTitle = '';
      this.customArticleContent = '';
      this.editingCustomArticleId = null;
    },
    async handleSaveCustomArticle() {
      const title = this.customArticleTitle.trim();
      const content = this.customArticleContent.trim();
      if (!title) {
        this.$message?.warning('请输入文章标题');
        return;
      }
      if (!content) {
        this.$message?.warning('请输入文章内容');
        return;
      }
      this.isSavingCustomArticle = true;
      const isEditing = this.editingCustomArticleId !== null;
      try {
        const payload = { title, content, difficulty: 'custom', lang: 'en' };
        if (isEditing) {
          await axios.put(`/api/custom-articles/${this.editingCustomArticleId}`, payload);
          this.$message?.success('自定义文章更新成功');
        } else {
          await axios.post('/api/custom-articles', payload);
          this.$message?.success('自定义文章添加成功');
        }
        const targetId = isEditing ? `custom-${this.editingCustomArticleId}` : null;
        this.closeCustomArticleModal();
        await this.loadData();
        if (isEditing && targetId) {
          const updated = this.englishTexts.find(t => t.id === targetId);
          if (updated) {
            this.selectedTextId = targetId;
            this.handleTextChange(targetId);
          }
        }
      } catch (error) {
        console.error(isEditing ? '更新自定义文章失败:' : '保存自定义文章失败:', error);
        this.$message?.error(isEditing ? '更新失败，请重试' : '保存失败，请重试');
      } finally {
        this.isSavingCustomArticle = false;
      }
    },
    handleDeleteCustomArticle() {
      if (!this.currentText || !this.currentText.isCustom) return;
      const realId = this.currentText.realId;
      const articleTitle = this.currentText.title;
      Modal.confirm({
        title: '删除自定义文章',
        content: `确定删除「${articleTitle}」吗？删除后不可恢复。`,
        okText: '删除',
        okType: 'danger',
        cancelText: '取消',
        onOk: async () => {
          try {
            await axios.delete(`/api/custom-articles/${realId}`);
            this.$message?.success('删除成功');
            await this.loadData();
          } catch (error) {
            console.error('删除自定义文章失败:', error);
            this.$message?.error('删除失败，请重试');
          }
        },
      });
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
      if (event.isComposing) return;
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