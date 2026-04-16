<template>
  <div class="wubi-graph-container">
    <div ref="graphRef" class="wubi-graph">
      <div v-if="wubiRoots.length > 0">
        <h3>字根关系图</h3>
        <p>字根数量: {{ wubiRoots.length }}</p>
        <div class="nodes-list">
          <div 
            v-for="(root, index) in wubiRoots" 
            :key="root.id"
            class="node-item"
            :style="{ backgroundColor: getNodeColor(index) }"
          >
            <div class="node-character">{{ root.character }}</div>
            <div class="node-code">{{ root.code }}</div>
            <div class="node-position">{{ root.position }}</div>
          </div>
        </div>
      </div>
      <div v-else>
        <p>暂无字根数据</p>
      </div>
    </div>
  </div>
</template>

<script>
export default {
  name: 'WubiGraph',
  props: {
    wubiRoots: {
      type: Array,
      default: () => []
    }
  },
  data() {
    return {
      graphRef: null
    };
  },
  mounted() {
    // 初始化图表（简单版本，后续可升级为G6）
    console.log('WubiGraph mounted with', this.wubiRoots.length, 'roots');
  },
  methods: {
    getNodeColor(index) {
      const colors = ['#ffebee', '#f3e5f5', '#e8eaf6', '#e0f2f1', '#fff3e0'];
      return colors[index % colors.length];
    }
  }
};
</script>

<style scoped>
.wubi-graph-container {
  width: 100%;
  height: 400px;
}

.wubi-graph {
  width: 100%;
  height: 100%;
  border: 1px solid #ddd;
  border-radius: 4px;
  padding: 16px;
  overflow-y: auto;
  background-color: #fafafa;
}

.nodes-list {
  display: flex;
  flex-wrap: wrap;
  gap: 12px;
  margin-top: 16px;
}

.node-item {
  padding: 12px;
  border-radius: 6px;
  border: 1px solid #ccc;
  min-width: 100px;
  text-align: center;
}

.node-character {
  font-size: 1.2em;
  font-weight: bold;
  color: #1890ff;
}

.node-code {
  font-family: monospace;
  font-size: 0.9em;
  color: #52c41a;
}

.node-position {
  font-size: 0.8em;
  color: #666;
}
</style>