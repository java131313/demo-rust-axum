<template>
  <div class="app-container">
    <a-layout style="min-height: 100vh;">
      <a-layout-header class="header">
        <div class="logo">五笔打字练习</div>
        <div class="user-info">
          <span v-if="user" class="username">{{ user.username }}</span>
          <a-button type="primary" ghost @click="handleLogout">退出登录</a-button>
        </div>
      </a-layout-header>
      <a-layout-content class="content">
        <a-tabs default-active-key="1">
          <a-tab-pane key="1" tab="五笔打字练习">
            <WubiTypingPractice />
          </a-tab-pane>
          <a-tab-pane key="3" tab="英语打字练习">
            <EnglishTypingPractice />
          </a-tab-pane>
          <a-tab-pane key="4" tab="拼音打字练习">
            <PinyinTypingPractice />
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
</template>

<script setup>
import { ref, onMounted } from 'vue';
import { message } from 'ant-design-vue';
import { useRouter } from 'vue-router';
import { logout } from '../api';
import WubiTypingPractice from './WubiTypingPractice.vue';
import WubiLookup from './WubiLookup.vue';
import EnglishTypingPractice from './EnglishTypingPractice.vue';
import PinyinTypingPractice from './PinyinTypingPractice.vue';

const router = useRouter();
const user = ref(null);

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
  max-width: 1200px;
  margin: 0 auto;
}

.header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 20px;
  background-color: #1890ff;
  color: white;
}

.logo {
  font-size: 1.5rem;
  font-weight: bold;
}

.user-info {
  display: flex;
  align-items: center;
  gap: 16px;
}

.username {
  font-size: 1rem;
}

.content {
  padding: 24px;
}

.footer {
  text-align: center;
  padding: 16px;
  background: #f0f2f5;
}
</style>
