<template>
  <div class="db-admin">
    <!-- 工具栏：连接状态 + 操作 -->
    <div class="db-toolbar">
      <span class="ws-dot" :class="wsStatus"></span>
      <span class="ws-text">{{ wsStatusText }}</span>
      <a-button size="small" @click="connect">重新连接</a-button>
      <span class="db-hint">通过 WebSocket 管理 SQLite 数据库 · 执行任意 SQL</span>
    </div>

    <div class="db-body">
      <!-- 左侧：表列表 -->
      <div class="db-sidebar">
        <div class="sidebar-header">
          <span>数据表 ({{ tables.length }})</span>
          <a-button size="small" type="link" @click="sendTables">刷新</a-button>
        </div>
        <div class="table-list">
          <div
            v-for="t in tables"
            :key="t"
            class="table-item"
            :class="{ active: t === currentTable && mode === 'browse' }"
            @click="loadTable(t)"
          >
            <span class="table-icon">▦</span> {{ t }}
          </div>
          <div v-if="!tables.length" class="sidebar-empty">暂无表</div>
        </div>
      </div>

      <!-- 右侧：SQL编辑区 + 结果网格 -->
      <div class="db-main">
        <div class="sql-area">
          <a-textarea
            v-model:value="sql"
            class="sql-input"
            placeholder="输入 SQL 语句，Ctrl+Enter 执行。例：SELECT * FROM articles LIMIT 10"
            :auto-size="{ minRows: 2, maxRows: 8 }"
            @keyup.ctrl.enter="runSql"
          />
          <div class="sql-actions">
            <a-button type="primary" :loading="running" @click="runSql">执行</a-button>
            <a-button v-if="mode === 'browse'" size="small" @click="loadSchema">查看建表语句</a-button>
          </div>
        </div>

        <a-alert v-if="errorMsg" type="error" :message="errorMsg" closable class="error-bar" @close="errorMsg = ''" />

        <div v-if="resultInfo" class="result-meta">
          {{ resultInfo.columns.length }} 列 · 返回 {{ resultInfo.rows.length }} 行
          <template v-if="resultInfo.rows_affected !== undefined && !resultInfo.columns.length">
            · 影响 {{ resultInfo.rows_affected }} 行
          </template>
          · 耗时 {{ resultInfo.elapsed_ms }}ms
          <template v-if="mode === 'browse'">
            <span class="pager">
              <a-button size="small" :disabled="offset === 0" @click="prevPage">上一页</a-button>
              <span class="pager-info">第 {{ offset / pageSize + 1 }} 页</span>
              <a-button size="small" :disabled="resultInfo.rows.length < pageSize" @click="nextPage">下一页</a-button>
            </span>
          </template>
        </div>

        <div v-if="resultInfo && resultInfo.columns.length" class="result-grid">
          <table>
            <thead>
              <tr>
                <th v-for="c in resultInfo.columns" :key="c">{{ c }}</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="(row, i) in resultInfo.rows" :key="i">
                <td
                  v-for="(cell, j) in row"
                  :key="j"
                  :class="{ 'cell-null': cell === null }"
                >{{ cell === null ? 'NULL' : cell }}</td>
              </tr>
            </tbody>
          </table>
        </div>

        <div v-else-if="!errorMsg" class="empty-tip">
          <template v-if="wsStatus === 'connected'">从左侧选择数据表浏览数据，或在上方输入 SQL 执行</template>
          <template v-else>WebSocket 未连接，点击"重新连接"</template>
        </div>
      </div>
    </div>
  </div>
</template>

<script>
export default {
  name: 'DatabaseAdmin',
  data() {
    return {
      ws: null,
      wsStatus: 'disconnected', // disconnected / connecting / connected / error
      tables: [],
      currentTable: '',
      mode: 'query', // browse=浏览表数据, query=自定义SQL
      sql: '',
      running: false,
      errorMsg: '',
      resultInfo: null,
      pageSize: 100,
      offset: 0,
    };
  },
  computed: {
    wsStatusText() {
      return {
        disconnected: '未连接',
        connecting: '连接中…',
        connected: '已连接',
        error: '连接失败',
      }[this.wsStatus];
    },
  },
  mounted() {
    this.connect();
  },
  beforeUnmount() {
    if (this.ws) {
      this.ws.close();
      this.ws = null;
    }
  },
  methods: {
    connect() {
      if (this.ws) {
        this.ws.close();
        this.ws = null;
      }
      const token = localStorage.getItem('token');
      if (!token) {
        this.errorMsg = '未登录，请先在首页登录后再使用数据库管理';
        this.wsStatus = 'error';
        return;
      }
      this.wsStatus = 'connecting';
      const proto = location.protocol === 'https:' ? 'wss' : 'ws';
      const ws = new WebSocket(`${proto}://${location.host}/ws/dbadmin?token=${encodeURIComponent(token)}`);
      this.ws = ws;

      ws.onopen = () => {
        this.wsStatus = 'connected';
        this.errorMsg = '';
        this.sendTables();
      };
      ws.onmessage = (event) => this.onMessage(event.data);
      ws.onclose = () => {
        if (this.ws === ws) this.wsStatus = 'disconnected';
      };
      ws.onerror = () => {
        if (this.ws === ws) this.wsStatus = 'error';
      };
    },
    send(obj) {
      if (!this.ws || this.ws.readyState !== WebSocket.OPEN) {
        this.errorMsg = '连接未就绪，请点击"重新连接"';
        return false;
      }
      this.ws.send(JSON.stringify(obj));
      return true;
    },
    sendTables() {
      this.send({ action: 'tables' });
    },
    loadTable(name) {
      this.currentTable = name;
      this.mode = 'browse';
      this.offset = 0;
      this.send({ action: 'rows', table: name, limit: this.pageSize, offset: 0 });
    },
    loadSchema() {
      if (!this.currentTable) return;
      this.send({ action: 'schema', table: this.currentTable });
    },
    runSql() {
      const sql = this.sql.trim();
      if (!sql) return;
      this.running = true;
      this.mode = 'query';
      this.send({ action: 'query', sql });
    },
    nextPage() {
      this.offset += this.pageSize;
      this.send({ action: 'rows', table: this.currentTable, limit: this.pageSize, offset: this.offset });
    },
    prevPage() {
      this.offset = Math.max(0, this.offset - this.pageSize);
      this.send({ action: 'rows', table: this.currentTable, limit: this.pageSize, offset: this.offset });
    },
    onMessage(raw) {
      this.running = false;
      let msg;
      try {
        msg = JSON.parse(raw);
      } catch {
        return;
      }
      if (!msg.ok) {
        this.errorMsg = msg.error || '未知错误';
        return;
      }
      this.errorMsg = '';
      switch (msg.action) {
        case 'tables':
          this.tables = msg.tables || [];
          break;
        case 'rows':
          this.resultInfo = msg;
          break;
        case 'query':
          this.resultInfo = msg;
          break;
        case 'schema':
          if (msg.sql) {
            this.sql = msg.sql;
            this.$message?.success(`已加载「${msg.table}」的建表语句到编辑器`);
          } else {
            this.$message?.warning('未找到建表语句');
          }
          break;
      }
    },
  },
};
</script>

<style scoped>
.db-admin {
  padding: 12px;
  background: #f5f5f5;
  border-radius: 8px;
}

.db-toolbar {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 10px;
  padding: 8px 12px;
  background: #fff;
  border-radius: 6px;
}

.ws-dot {
  width: 10px;
  height: 10px;
  border-radius: 50%;
  background: #bbb;
}
.ws-dot.connected { background: #52c41a; }
.ws-dot.connecting { background: #faad14; }
.ws-dot.error { background: #f5222d; }

.ws-text { font-size: 13px; color: #666; min-width: 48px; }
.db-hint { color: #999; font-size: 12px; margin-left: auto; }

.db-body {
  display: flex;
  gap: 10px;
  min-height: 480px;
}

.db-sidebar {
  width: 220px;
  flex-shrink: 0;
  background: #fff;
  border-radius: 6px;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.sidebar-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px 12px;
  border-bottom: 1px solid #f0f0f0;
  font-weight: 600;
  font-size: 13px;
}

.table-list {
  overflow-y: auto;
  flex: 1;
  max-height: 560px;
}

.table-item {
  padding: 7px 12px;
  cursor: pointer;
  font-size: 13px;
  font-family: monospace;
  color: #333;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.table-item:hover { background: #f0f5ff; }
.table-item.active { background: #e6f4ff; color: #1677ff; }
.table-icon { color: #1677ff; margin-right: 4px; }
.sidebar-empty { padding: 20px; text-align: center; color: #999; font-size: 12px; }

.db-main {
  flex: 1;
  background: #fff;
  border-radius: 6px;
  padding: 12px;
  display: flex;
  flex-direction: column;
  min-width: 0;
}

.sql-area { display: flex; gap: 8px; align-items: flex-start; }
.sql-input { flex: 1; font-family: monospace; font-size: 13px; }
.sql-actions { display: flex; flex-direction: column; gap: 6px; }

.error-bar { margin: 8px 0; }

.result-meta {
  margin: 10px 0 6px;
  font-size: 12px;
  color: #666;
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}
.pager { margin-left: auto; display: flex; align-items: center; gap: 6px; }
.pager-info { color: #999; }

.result-grid {
  overflow: auto;
  max-height: 460px;
  border: 1px solid #f0f0f0;
  border-radius: 4px;
  flex: 1;
}

.result-grid table {
  border-collapse: collapse;
  width: 100%;
  font-size: 13px;
}

.result-grid th, .result-grid td {
  border: 1px solid #f0f0f0;
  padding: 5px 10px;
  text-align: left;
  white-space: nowrap;
  max-width: 320px;
  overflow: hidden;
  text-overflow: ellipsis;
}

.result-grid th {
  background: #fafafa;
  position: sticky;
  top: 0;
  font-family: monospace;
  color: #1677ff;
}

.result-grid tbody tr:nth-child(even) { background: #fcfcfc; }
.result-grid tbody tr:hover { background: #f0f5ff; }
.cell-null { color: #c41d7f; font-style: italic; }

.empty-tip {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  color: #bbb;
  font-size: 13px;
}

@media (max-width: 768px) {
  .db-body { flex-direction: column; }
  .db-sidebar { width: 100%; }
  .table-list { max-height: 180px; }
}
</style>
