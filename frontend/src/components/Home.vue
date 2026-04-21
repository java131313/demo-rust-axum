<template>
  <div class="app-container">
    <div class="floating-content">
      <a-layout style="min-height: 100vh;">
        <a-layout-header class="header">
          <div class="logo">五笔打字练习</div>
          <div class="user-info">
          <span v-if="user" class="username">{{ user.username }}</span>
          <a-switch 
            v-model:checked="keyboardEnabled" 
            @change="toggleKeyboard"
            style="margin-right: 16px;"
          />
          <span style="margin-right: 16px;">{{ keyboardEnabled ? '键盘已开启' : '键盘已关闭' }}</span>
          <a-button type="primary" ghost @click="handleLogout">退出登录</a-button>
        </div>
        </a-layout-header>
        <a-layout-content class="content">
          <a-tabs v-model:activeKey="activeTabKey" @change="onTabChange">
            <a-tab-pane key="1" tab="五笔打字练习">
              <WubiTypingPractice />
            </a-tab-pane>
            <a-tab-pane key="3" tab="英语打字练习">
              <EnglishTypingPractice />
            </a-tab-pane>
            <a-tab-pane key="4" tab="拼音打字练习">
              <PinyinTypingPractice />
            </a-tab-pane>
            <a-tab-pane key="5" tab="字根键盘与口诀">
              <WubiRootKeyboard />
            </a-tab-pane>
            <a-tab-pane key="2" tab="汉字五笔查询">
              <WubiLookup />
            </a-tab-pane>
          </a-tabs>
        </a-layout-content>
        <a-layout-footer class="footer">
          五笔打字练习系统 ©2024 - 后端接口：demo-rust-axum
        </a-layout-footer>
      </a-layout>
    </div>

    <!-- 虚拟键盘层 -->
    <div 
      v-if="keyboardVisible" 
      class="keyboard-container"
      :style="keyboardPosition"
      @mousedown="dragStart"
    >
      <div class="keyboard-content">
        <VirtualKeyboard
          :active-key="keyboard.activeKey"
          :wubi-code="keyboard.wubiCode"
          :code-index="keyboard.codeIndex"
        />
        <div class="keyboard-close" @click="hideKeyboard">
          <span>×</span>
        </div>
        <div class="keyboard-handle">
          <span>拖拽移动</span>
        </div>
        <div class="keyboard-resize-handle" @mousedown="resizeStart">
          <div class="resize-icon"></div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, reactive, provide, nextTick, onMounted } from 'vue';
import { message } from 'ant-design-vue';
import { useRouter } from 'vue-router';
import { logout } from '../api';
import VirtualKeyboard from './VirtualKeyboard.vue';
import WubiTypingPractice from './WubiTypingPractice.vue';
import WubiLookup from './WubiLookup.vue';
import EnglishTypingPractice from './EnglishTypingPractice.vue';
import PinyinTypingPractice from './PinyinTypingPractice.vue';
import WubiRootKeyboard from './WubiRootKeyboard.vue';

const router = useRouter();
const user = ref(null);

const activeTabKey = ref('1');
const keyboardEnabled = ref(false);
const keyboardVisible = ref(false);
const keyboard = reactive({
  activeKey: null,
  wubiCode: null,
  codeIndex: 0,
});

// 拖拽相关状态
const isDragging = ref(false);
const isResizing = ref(false);
const dragOffset = reactive({
  x: 0,
  y: 0,
});
const resizeOffset = reactive({
  x: 0,
  y: 0,
});
const keyboardPosition = reactive({
  bottom: '20px',
  right: '20px',
  width: '600px',
  minWidth: '400px',
  minHeight: '150px',
});
const keyboardSyncByTab = new Map();

function toggleKeyboard(checked) {
  keyboardEnabled.value = checked;
  if (!checked) {
    keyboardVisible.value = false;
  }
}

function setKeyboard(partial) {
  Object.assign(keyboard, partial);
  if (keyboardEnabled.value) {
    keyboardVisible.value = true;
  }
}

function resetKeyboard() {
  keyboard.activeKey = null;
  keyboard.wubiCode = null;
  keyboard.codeIndex = 0;
  keyboardVisible.value = false;
}

function showKeyboard() {
  if (keyboardEnabled.value) {
    keyboardVisible.value = true;
  }
}

function hideKeyboard() {
  keyboardVisible.value = false;
}

// 拖拽相关方法
function dragStart(event) {
  // 如果正在调整大小，不触发拖拽
  if (isResizing.value) return;
  
  isDragging.value = true;
  const keyboardEl = event.currentTarget;
  const rect = keyboardEl.getBoundingClientRect();
  
  // 计算鼠标相对于键盘元素的偏移
  dragOffset.x = event.clientX - rect.left;
  dragOffset.y = event.clientY - rect.top;
  
  // 移除默认的transform，改为使用top和left
  keyboardPosition.transform = 'none';
  keyboardPosition.bottom = 'auto';
  keyboardPosition.left = rect.left + 'px';
  keyboardPosition.top = rect.top + 'px';
  
  // 添加鼠标移动和释放事件监听
  document.addEventListener('mousemove', dragMove);
  document.addEventListener('mouseup', dragEnd);
}

function dragMove(event) {
  if (!isDragging.value) return;
  
  // 计算新位置
  const newX = event.clientX - dragOffset.x;
  const newY = event.clientY - dragOffset.y;
  
  // 限制在视口内
  const viewportWidth = window.innerWidth;
  const viewportHeight = window.innerHeight;
  const keyboardWidth = 600; // 键盘宽度
  const keyboardHeight = 200; // 键盘高度
  
  const clampedX = Math.max(0, Math.min(newX, viewportWidth - keyboardWidth));
  const clampedY = Math.max(0, Math.min(newY, viewportHeight - keyboardHeight));
  
  keyboardPosition.left = clampedX + 'px';
  keyboardPosition.top = clampedY + 'px';
}

function dragEnd() {
  isDragging.value = false;
  // 移除事件监听
  document.removeEventListener('mousemove', dragMove);
  document.removeEventListener('mouseup', dragEnd);
}

// 调整大小相关方法
function resizeStart(event) {
  isResizing.value = true;
  const keyboardEl = event.currentTarget.parentElement;
  const rect = keyboardEl.getBoundingClientRect();
  
  // 计算鼠标相对于键盘右下角的偏移
  resizeOffset.x = event.clientX - rect.right;
  resizeOffset.y = event.clientY - rect.bottom;
  
  // 添加鼠标移动和释放事件监听
  document.addEventListener('mousemove', resizeMove);
  document.addEventListener('mouseup', resizeEnd);
  
  // 阻止默认行为和冒泡
  event.preventDefault();
  event.stopPropagation();
}

function resizeMove(event) {
  if (!isResizing.value) return;
  
  // 计算新的宽度和高度
  const newWidth = event.clientX - resizeOffset.x;
  const newHeight = event.clientY - resizeOffset.y;
  
  // 限制最小尺寸
  const minWidth = 400;
  const minHeight = 150;
  
  const clampedWidth = Math.max(minWidth, newWidth);
  const clampedHeight = Math.max(minHeight, newHeight);
  
  keyboardPosition.width = clampedWidth + 'px';
  keyboardPosition.height = clampedHeight + 'px';
  
  // 阻止默认行为
  event.preventDefault();
}

function resizeEnd() {
  isResizing.value = false;
  // 移除事件监听
  document.removeEventListener('mousemove', resizeMove);
  document.removeEventListener('mouseup', resizeEnd);
}

function registerKeyboardSync(tabKey, fn) {
  keyboardSyncByTab.set(tabKey, fn);
  return () => {
    keyboardSyncByTab.delete(tabKey);
  };
}

const homeTabs = { activeTabKey };
provide('homeTabs', homeTabs);
provide('virtualKeyboard', {
  setKeyboard,
  resetKeyboard,
  registerKeyboardSync,
  showKeyboard,
  hideKeyboard,
  keyboardVisible,
  keyboardEnabled,
});

function onTabChange(key) {
  resetKeyboard();
  nextTick(() => {
    const sync = keyboardSyncByTab.get(key);
    if (typeof sync === 'function') sync();
  });
}

onMounted(() => {
  const userStr = localStorage.getItem('user');
  if (userStr) {
    user.value = JSON.parse(userStr);
  }
});

const handleLogout = async () => {
  try {
    await logout();
    localStorage.removeItem('token');
    localStorage.removeItem('user');
    user.value = null;
    message.success('退出成功');
    router.push('/login');
  } catch (error) {
    message.error('退出失败');
    // 即使API调用失败，也要清理本地存储
    localStorage.removeItem('token');
    localStorage.removeItem('user');
    user.value = null;
    router.push('/login');
  }
};
</script>

<style scoped>
.app-container {
  max-width: 1280px;
  margin: 0 auto;
  position: relative;
  min-height: 100vh;
}

.floating-content {
  position: relative;
  z-index: 1;
  margin-bottom: 220px;
}

.header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  flex-wrap: wrap;
  gap: 12px;
  padding: 12px 20px;
  background-color: #1677ff;
  color: white;
}

.logo {
  font-size: clamp(1.35rem, 2.5vw, 1.75rem);
  font-weight: 700;
  letter-spacing: 0.03em;
}

.user-info {
  display: flex;
  align-items: center;
  gap: 16px;
  flex-wrap: wrap;
}

.username {
  font-size: 1.1rem;
  font-weight: 500;
}

.content {
  padding: 20px 16px 28px;
}

/* 全局虚拟键盘包装器 */
.global-virtual-keyboard-wrap {
  margin-bottom: 0;
}

/* 虚拟键盘 */
.keyboard-container {
  position: fixed;
  z-index: 1000;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.1);
  border-radius: 12px;
  background: transparent;
  backdrop-filter: blur(10px);
  border: 1px solid rgba(255, 255, 255, 0.3);
  max-width: 90vw;
  animation: slide-up 0.3s ease-out;
}

.keyboard-content {
  position: relative;
  padding: 16px;
  cursor: move;
  background: transparent;
  border-radius: 8px;
}

.keyboard-handle {
  position: absolute;
  top: 4px;
  left: 16px;
  font-size: 12px;
  color: #666;
  background: rgba(255, 255, 255, 0.8);
  padding: 2px 8px;
  border-radius: 4px;
  cursor: move;
  backdrop-filter: blur(5px);
}

.keyboard-handle:hover {
  background: rgba(255, 255, 255, 0.9);
  color: #333;
}

.keyboard-resize-handle {
  position: absolute;
  bottom: 8px;
  right: 8px;
  width: 16px;
  height: 16px;
  cursor: nwse-resize;
  background: rgba(255, 255, 255, 0.8);
  border: 1px solid rgba(255, 255, 255, 0.5);
  border-radius: 2px;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s ease;
  backdrop-filter: blur(5px);
}

.keyboard-resize-handle:hover {
  background: rgba(255, 255, 255, 0.9);
  border-color: #1677ff;
}

.resize-icon {
  width: 8px;
  height: 8px;
  background: #666;
  clip-path: polygon(100% 0, 0 100%, 100% 100%);
}

.keyboard-close {
  position: absolute;
  top: 8px;
  right: 8px;
  width: 24px;
  height: 24px;
  border-radius: 50%;
  background: rgba(255, 255, 255, 0.8);
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  font-size: 16px;
  color: #666;
  transition: all 0.2s ease;
  backdrop-filter: blur(5px);
}

.keyboard-close:hover {
  background: rgba(255, 255, 255, 0.9);
  color: #333;
}

.keyboard-container :deep(.virtual-keyboard) {
  margin: 0;
  width: 100%;
  height: 100%;
  min-height: 150px;
  background: transparent;
}

.keyboard-container :deep(.keyboard-header) {
  background: transparent;
  color: rgba(255, 255, 255, 0.9);
}

.keyboard-container :deep(.keyboard-header p) {
  color: rgba(255, 255, 255, 0.7);
}

.keyboard-container :deep(.code-display) {
  background: transparent;
}

.keyboard-container :deep(.code-char) {
  background: rgba(255, 255, 255, 0.2);
  border: 1px solid rgba(255, 255, 255, 0.3);
  color: rgba(255, 255, 255, 0.9);
}

.keyboard-container :deep(.code-char.current) {
  background: rgba(251, 191, 36, 0.7);
  border-color: rgba(251, 191, 36, 0.9);
  color: rgba(30, 41, 59, 0.9);
}

.keyboard-container :deep(.keyboard) {
  background: transparent;
}

.keyboard-container :deep(.key) {
  background: rgba(255, 255, 255, 0.2);
  border: 1px solid rgba(255, 255, 255, 0.3);
  color: rgba(255, 255, 255, 0.9);
  backdrop-filter: blur(5px);
}

.keyboard-container :deep(.key:hover) {
  background: rgba(255, 255, 255, 0.3);
}

.keyboard-container :deep(.key.pinky) {
  background: rgba(124, 58, 237, 0.3);
  border-color: rgba(139, 92, 246, 0.5);
}

.keyboard-container :deep(.key.ring) {
  background: rgba(37, 99, 235, 0.3);
  border-color: rgba(59, 130, 246, 0.5);
}

.keyboard-container :deep(.key.middle) {
  background: rgba(5, 150, 105, 0.3);
  border-color: rgba(16, 185, 129, 0.5);
}

.keyboard-container :deep(.key.index-left) {
  background: rgba(217, 119, 6, 0.3);
  border-color: rgba(245, 158, 11, 0.5);
}

.keyboard-container :deep(.key.index-right) {
  background: rgba(220, 38, 38, 0.3);
  border-color: rgba(239, 68, 68, 0.5);
}

.keyboard-container :deep(.key.thumb) {
  background: rgba(8, 145, 178, 0.3);
  border-color: rgba(6, 182, 212, 0.5);
}

.keyboard-container :deep(.key.active) {
  background: rgba(251, 191, 36, 0.7) !important;
  border-color: rgba(251, 191, 36, 0.9) !important;
  color: rgba(30, 41, 59, 0.9) !important;
  backdrop-filter: blur(5px);
}

.keyboard-container :deep(.radical-tooltip) {
  background: rgba(15, 23, 42, 0.8);
  border: 1px solid rgba(71, 85, 105, 0.5);
  backdrop-filter: blur(10px);
}

.keyboard-container :deep(.tooltip-key) {
  color: rgba(251, 191, 36, 0.9);
}

.keyboard-container :deep(.tooltip-radicals) {
  color: rgba(226, 232, 240, 0.9);
}

.keyboard-container :deep(.tooltip-desc) {
  color: rgba(203, 213, 225, 0.7);
}

.keyboard-container :deep(.finger-legend) {
  background: transparent;
  border-top: 1px solid rgba(255, 255, 255, 0.2);
}

.keyboard-container :deep(.legend-item) {
  color: rgba(203, 213, 225, 0.7);
}

/* 动画效果 */
@keyframes slide-up {
  from {
    opacity: 0;
    transform: translate(100%, 100%);
  }
  to {
    opacity: 1;
    transform: translate(0, 0);
  }
}

/* 响应式调整 */
@media (max-width: 768px) {
  .keyboard-container {
    width: 95vw;
    right: 10px;
    bottom: 10px;
  }
  
  .keyboard-content {
    padding: 12px;
  }
}

.footer {
  text-align: center;
  padding: 18px 16px;
  font-size: 1rem;
  line-height: 1.6;
  color: #434343;
  background: #f0f2f5;
}
</style>
