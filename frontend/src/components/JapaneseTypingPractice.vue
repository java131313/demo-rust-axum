<template>
  <div class="japanese-typing-practice">
    <a-card title="日语打字练习" class="typing-card">
      <div class="practice-content">
        <div class="tab-container">
          <a-tabs v-model:activeKey="selectedTab" @change="handleTabChange">
            <a-tab-pane key="hiragana" tab="平假名"></a-tab-pane>
            <a-tab-pane key="katakana" tab="片假名"></a-tab-pane>
            <a-tab-pane key="kanji" tab="汉字"></a-tab-pane>
            <a-tab-pane key="mixed" tab="混合"></a-tab-pane>
          </a-tabs>
        </div>
        <div class="text-container">
          <h3>{{ currentText.title }}</h3>
          <div class="text-display">
            <span 
              v-for="(char, index) in currentText.content" 
              :key="index"
              :class="{
                'correct': index < correctCount,
                'current': index === correctCount,
                'remaining': index > correctCount
              }"
            >
              {{ char }}
            </span>
          </div>
        </div>
        
        <div class="input-area">
          <a-input
            v-model:value="userInput"
            placeholder="请输入日语文字"
            @input="handleInput"
            @keyup.enter="nextText"
            size="large"
            :disabled="isCompleted"
          />
          <a-button 
            type="primary" 
            @click="nextText"
            :disabled="!isCompleted"
            style="margin-top: 12px"
          >
            下一篇
          </a-button>
        </div>
        
        <div class="stats">
          <div class="stat-item">
            <span class="stat-label">正确率:</span>
            <span class="stat-value">{{ accuracy }}%</span>
          </div>
          <div class="stat-item">
            <span class="stat-label">速度:</span>
            <span class="stat-value">{{ speed }} 字符/分钟</span>
          </div>
          <div class="stat-item">
            <span class="stat-label">已完成:</span>
            <span class="stat-value">{{ completedCount }}/{{ totalTexts }}</span>
          </div>
        </div>
        
        <div class="practice-controls">
          <div class="difficulty-selector">
            <span>难度选择:</span>
            <a-radio-group v-model:value="selectedDifficulty" @change="changeDifficulty">
              <a-radio-button value="easy">简单</a-radio-button>
              <a-radio-button value="medium">中等</a-radio-button>
              <a-radio-button value="hard">困难</a-radio-button>
            </a-radio-group>
          </div>
          <div class="mode-selector">
            <span>模式选择:</span>
            <a-radio-group v-model:value="selectedMode" @change="handleModeChange">
              <a-radio-button value="manual">手动打字</a-radio-button>
              <a-radio-button value="auto">自动打字</a-radio-button>
            </a-radio-group>
          </div>
        </div>
      </div>
    </a-card>
  </div>
</template>

<script setup>
import { ref, onMounted, watch, computed, inject, nextTick } from 'vue';
import { message } from 'ant-design-vue';
import { getJapaneseTexts } from '../api';

// 注入虚拟键盘相关功能
const { setKeyboard, registerKeyboardSync } = inject('virtualKeyboard', {});
const { activeTabKey } = inject('homeTabs', {});

const userInput = ref('');
const japaneseTexts = ref([]);
const originalJapaneseTexts = ref([]);
const currentTextIndex = ref(0);
const correctCount = ref(0);
const startTime = ref(Date.now());
const completedCount = ref(0);
const selectedDifficulty = ref('easy');
const selectedMode = ref('manual');
const selectedTab = ref('hiragana');
const autoTimer = ref(null);
const isLoading = ref(true);

const currentText = computed(() => {
  if (japaneseTexts.value.length === 0) {
    return { title: '加载中...', content: '' };
  }
  return japaneseTexts.value[currentTextIndex.value] || japaneseTexts.value[0];
});

const totalTexts = computed(() => japaneseTexts.value.length);

const isCompleted = computed(() => {
  return correctCount.value >= currentText.value.content.length;
});

const accuracy = computed(() => {
  if (userInput.value.length === 0) return 100;
  let correct = 0;
  for (let i = 0; i < userInput.value.length; i++) {
    if (i < currentText.value.content.length && userInput.value[i] === currentText.value.content[i]) {
      correct++;
    }
  }
  return Math.round((correct / userInput.value.length) * 100);
});

const speed = computed(() => {
  const elapsedTime = (Date.now() - startTime.value) / 60000; // 转换为分钟
  if (elapsedTime === 0) return 0;
  return Math.round(correctCount.value / elapsedTime);
});

const loadJapaneseTexts = async () => {
  try {
    isLoading.value = true;
    const response = await getJapaneseTexts();
    originalJapaneseTexts.value = response.data;
    if (originalJapaneseTexts.value.length > 0) {
      filterTextsByTab();
    }
  } catch (error) {
    message.error('加载日语文章失败');
    console.error('Error loading Japanese texts:', error);
  } finally {
    isLoading.value = false;
  }
};

const filterTextsByTab = () => {
  let filtered = [...originalJapaneseTexts.value];
  
  // 首先根据难度过滤
  if (selectedDifficulty.value !== 'all') {
    filtered = filtered.filter(text => text.difficulty === selectedDifficulty.value);
  }
  
  // 然后根据选中的tab过滤
  switch (selectedTab.value) {
    case 'hiragana':
      // 只包含平假名
      filtered = filtered.filter(text => text.type === 'hiragana');
      break;
    case 'katakana':
      // 只包含片假名
      filtered = filtered.filter(text => text.type === 'katakana');
      break;
    case 'kanji':
      // 只包含汉字
      filtered = filtered.filter(text => text.type === 'kanji');
      break;
    case 'mixed':
      // 包含混合内容
      filtered = filtered.filter(text => text.type === 'mixed');
      break;
  }
  
  japaneseTexts.value = filtered;
  currentTextIndex.value = 0;
  correctCount.value = 0;
  userInput.value = '';
  startTime.value = Date.now();
};

const handleTabChange = () => {
  filterTextsByTab();
};

const filterTextsByDifficulty = () => {
  if (selectedDifficulty.value === 'all') {
    return;
  }
  const filtered = japaneseTexts.value.filter(text => text.difficulty === selectedDifficulty.value);
  if (filtered.length > 0) {
    japaneseTexts.value = filtered;
  }
};

const changeDifficulty = () => {
  filterTextsByTab();
};

const handleInput = () => {
  const input = userInput.value;
  const text = currentText.value.content;
  
  let newCorrectCount = 0;
  for (let i = 0; i < input.length; i++) {
    if (i < text.length && input[i] === text[i]) {
      newCorrectCount = i + 1;
    } else {
      break;
    }
  }
  
  correctCount.value = newCorrectCount;
  
  // 更新虚拟键盘状态
  if (correctCount.value < text.length) {
    const nextChar = text[correctCount.value];
    setKeyboard({ activeKey: nextChar });
  } else {
    setKeyboard({ activeKey: null });
  }
  
  if (correctCount.value >= text.length) {
    completedCount.value++;
  }
};

const nextText = () => {
  if (currentTextIndex.value < japaneseTexts.value.length - 1) {
    currentTextIndex.value++;
  } else {
    currentTextIndex.value = 0;
  }
  correctCount.value = 0;
  userInput.value = '';
  startTime.value = Date.now();
  
  // 清除自动打字定时器
  if (autoTimer.value) {
    clearInterval(autoTimer.value);
    autoTimer.value = null;
  }
  
  // 更新虚拟键盘状态
  if (japaneseTexts.value.length > 0) {
    const firstChar = currentText.value.content[0];
    setKeyboard({ activeKey: firstChar });
  }
  
  // 如果是自动打字模式，开始自动打字
  if (selectedMode.value === 'auto') {
    startAutoTyping();
  }
};

const handleModeChange = () => {
  // 重置练习状态
  correctCount.value = 0;
  userInput.value = '';
  startTime.value = Date.now();
  
  // 清除自动打字定时器
  if (autoTimer.value) {
    clearInterval(autoTimer.value);
    autoTimer.value = null;
  }
  
  // 如果是自动打字模式，开始自动打字
  if (selectedMode.value === 'auto') {
    startAutoTyping();
  }
};

const startAutoTyping = () => {
  // 清除之前的定时器
  if (autoTimer.value) {
    clearInterval(autoTimer.value);
  }
  
  // 设置自动打字定时器，每3000毫秒输入一个字符（默认很慢的速度）
  autoTimer.value = setInterval(() => {
    if (correctCount.value < currentText.value.content.length) {
      const nextChar = currentText.value.content[correctCount.value];
      userInput.value += nextChar;
      
      // 更新正确计数
      correctCount.value++;
      
      // 更新虚拟键盘状态
      if (correctCount.value < currentText.value.content.length) {
        const nextNextChar = currentText.value.content[correctCount.value];
        setKeyboard({ activeKey: nextNextChar });
      } else {
        setKeyboard({ activeKey: null });
        completedCount.value++;
      }
    } else {
      // 清除定时器
      if (autoTimer.value) {
        clearInterval(autoTimer.value);
        autoTimer.value = null;
      }
    }
  }, 3000);
};

onMounted(() => {
  loadJapaneseTexts();
  
  // 注册键盘同步函数
  if (registerKeyboardSync) {
    registerKeyboardSync('6', () => {
      nextTick(() => {
        if (japaneseTexts.value.length > 0) {
          const firstChar = currentText.value.content[0];
          setKeyboard({ activeKey: firstChar });
        }
      });
    });
  }
});
</script>

<style scoped>
.japanese-typing-practice {
  padding: 20px;
}

.typing-card {
  max-width: 800px;
  margin: 0 auto;
}

.practice-content {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.tab-container {
  margin-bottom: 10px;
}

.text-container {
  background-color: #f5f5f5;
  padding: 20px;
  border-radius: 8px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

.text-container h3 {
  margin-bottom: 12px;
  color: #333;
  font-size: 18px;
}

.text-display {
  font-size: 24px;
  line-height: 1.5;
  font-family: 'Helvetica Neue', Arial, sans-serif;
  letter-spacing: 2px;
}

.text-display span {
  display: inline-block;
  min-width: 20px;
  text-align: center;
  margin: 0 2px;
}

.text-display .correct {
  color: #52c41a;
  text-decoration: underline;
}

.text-display .current {
  background-color: #1890ff;
  color: white;
  border-radius: 4px;
  padding: 0 4px;
}

.text-display .remaining {
  color: #999;
}

.input-area {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.stats {
  display: flex;
  gap: 24px;
  padding: 16px;
  background-color: #f0f2f5;
  border-radius: 8px;
}

.stat-item {
  display: flex;
  align-items: center;
  gap: 8px;
}

.stat-label {
  font-weight: 500;
  color: #666;
}

.stat-value {
  font-weight: 700;
  color: #1890ff;
}

.practice-controls {
  display: flex;
  flex-wrap: wrap;
  gap: 20px;
  padding: 16px;
  background-color: #f9f9f9;
  border-radius: 8px;
}

.difficulty-selector {
  display: flex;
  align-items: center;
  gap: 16px;
}

.mode-selector {
  display: flex;
  align-items: center;
  gap: 16px;
}

.difficulty-selector span,
.mode-selector span {
  font-weight: 500;
  color: #666;
}

@media (max-width: 768px) {
  .japanese-typing-practice {
    padding: 10px;
  }
  
  .text-display {
    font-size: 18px;
  }
  
  .stats {
    flex-direction: column;
    gap: 12px;
  }
  
  .practice-controls {
    flex-direction: column;
    align-items: flex-start;
  }
  
  .difficulty-selector,
  .mode-selector {
    flex-direction: column;
    align-items: flex-start;
    width: 100%;
  }
}
</style>