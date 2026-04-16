<template>
  <div class="app-container">
    <a-layout style="min-height: 100vh;">
      <a-layout-header class="header">
        <div class="logo">五笔打字练习</div>
      </a-layout-header>
      <a-layout-content class="content">
        <a-tabs default-active-key="1">
          <a-tab-pane key="1" tab="五笔打字练习">
            <WubiTypingPractice />
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

<script>
import WubiTypingPractice from './components/WubiTypingPractice.vue';
import WubiLookup from './components/WubiLookup.vue';

export default {
  name: 'App',
  components: {
    WubiTypingPractice,
    WubiLookup
  }
}
</script>

<style>
#app {
  font-family: Avenir, Helvetica, Arial, sans-serif;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
}

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

.content {
  padding: 24px;
}

.footer {
  text-align: center;
  padding: 16px;
  background: #f0f2f5;
}
</style>