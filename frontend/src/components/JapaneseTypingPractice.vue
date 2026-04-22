<template>
  <div class="japanese-typing-practice">
    <a-card title="日语打字练习" class="typing-card">
      <div class="practice-content">
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
        
        <div class="difficulty-selector">
          <span>难度选择:</span>
          <a-radio-group v-model:value="selectedDifficulty" @change="changeDifficulty">
            <a-radio-button value="easy">简单</a-radio-button>
            <a-radio-button value="medium">中等</a-radio-button>
            <a-radio-button value="hard">困难</a-radio-button>
          </a-radio-group>
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
const currentTextIndex = ref(0);
const correctCount = ref(0);
const startTime = ref(Date.now());
const completedCount = ref(0);
const selectedDifficulty = ref('easy');
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
    japaneseTexts.value = response.data;
    if (japaneseTexts.value.length > 0) {
      filterTextsByDifficulty();
    }
  } catch (error) {
    message.error('加载日语文章失败');
    console.error('Error loading Japanese texts:', error);
  } finally {
    isLoading.value = false;
  }
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
  currentTextIndex.value = 0;
  correctCount.value = 0;
  userInput.value = '';
  startTime.value = Date.now();
  loadJapaneseTexts();
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
  
  // 更新虚拟键盘状态
  if (japaneseTexts.value.length > 0) {
    const firstChar = currentText.value.content[0];
    setKeyboard({ activeKey: firstChar });
  }
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

.difficulty-selector {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 16px;
  background-color: #f9f9f9;
  border-radius: 8px;
}

.difficulty-selector span {
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
  
  .difficulty-selector {
    flex-direction: column;
    align-items: flex-start;
  }
}
</style>