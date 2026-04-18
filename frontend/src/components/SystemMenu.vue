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
  margin-bottom: 30px;
  font-size: 2rem;
}

.menu-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
  gap: 20px;
  padding: 20px;
}

.menu-card {
  background: white;
  border-radius: 15px;
  padding: 30px;
  text-align: center;
  cursor: pointer;
  transition: all 0.3s ease;
  box-shadow: 0 10px 30px rgba(0, 0, 0, 0.1);
}

.menu-card:hover {
  transform: translateY(-5px);
  box-shadow: 0 15px 40px rgba(0, 0, 0, 0.2);
}

.menu-icon {
  font-size: 3rem;
  margin-bottom: 15px;
}

.menu-title {
  font-size: 1.3rem;
  font-weight: bold;
  color: #333;
  margin-bottom: 10px;
}

.menu-desc {
  color: #666;
  font-size: 0.9rem;
}
</style>
