<template>
  <div class="system-menu-container">
    <h1>系统管理</h1>
    <div class="menu-grid">
      <div class="menu-card" @click="navigateTo('/articles')">
        <div class="menu-icon">📝</div>
        <div class="menu-title">文章管理</div>
        <div class="menu-desc">创建、编辑、删除文章</div>
      </div>
      <div class="menu-card" @click="navigateTo('/wubi-code')">
        <div class="menu-icon">🔤</div>
        <div class="menu-title">编码管理</div>
        <div class="menu-desc">修改汉字五笔编码</div>
      </div>
      <div class="menu-card" @click="navigateTo('/')">
        <div class="menu-icon">⌨️</div>
        <div class="menu-title">练习模式</div>
        <div class="menu-desc">返回打字练习</div>
      </div>
      <div class="menu-card" @click="navigateTo('/lookup')">
        <div class="menu-icon">🔍</div>
        <div class="menu-title">编码查询</div>
        <div class="menu-desc">查询汉字五笔编码</div>
      </div>
    </div>
  </div>
</template>

<script>
export default {
  name: 'SystemMenu',
  methods: {
    navigateTo(path) {
      this.$router.push(path)
    }
  }
}
</script>

<style scoped>
.system-menu-container {
  max-width: 1200px;
  margin: 0 auto;
  padding: 20px;
  min-height: 100vh;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
}

h1 {
  text-align: center;
  color: white;
  margin-bottom: 28px;
  font-size: clamp(1.75rem, 4vw, 2.25rem);
  font-weight: 700;
  text-shadow: 0 1px 2px rgba(0, 0, 0, 0.25);
}

.menu-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
  gap: 22px;
  padding: 16px;
}

.menu-card {
  background: white;
  border-radius: 16px;
  padding: 28px 24px;
  min-height: 200px;
  text-align: center;
  cursor: pointer;
  transition: box-shadow 0.2s ease, border-color 0.2s ease;
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.12);
  border: 2px solid transparent;
}

.menu-card:hover,
.menu-card:focus-visible {
  outline: none;
  border-color: #1677ff;
  box-shadow: 0 12px 32px rgba(22, 119, 255, 0.2);
}

.menu-icon {
  font-size: clamp(2.5rem, 6vw, 3.25rem);
  margin-bottom: 16px;
  line-height: 1;
}

.menu-title {
  font-size: 1.35rem;
  font-weight: 700;
  color: #1a1a1a;
  margin-bottom: 12px;
  line-height: 1.35;
}

.menu-desc {
  color: #434343;
  font-size: 1.05rem;
  line-height: 1.55;
}
</style>
