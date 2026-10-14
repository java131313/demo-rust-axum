<template>
  <div class="db-admin">
    <!-- 工具栏：连接状态 + 操作 -->
    <div class="db-toolbar">
      <span class="ws-dot" :class="wsStatus"></span>
      <span class="ws-text">{{ wsStatusText }}</span>
      <a-button size="small" @click="connect">重新连接</a-button>
      <a-button
        v-if="mode === 'browse'"
        size="small"
        type="primary"
        @click="startAddRow"
      >+ 新增一行</a-button>
      <a-button size="small" @click="showAuditLog">修改记录</a-button>
      <span class="db-hint">双击单元格可编辑 · 按行保存 · 执行任意 SQL</span>
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

      <!-- 右侧：SQL编辑区 + 可编辑结果网格 -->
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
                <th v-if="mode === 'browse'" class="op-col">操作</th>
                <th v-for="c in resultInfo.columns" :key="c">{{ c }}</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="(row, i) in resultInfo.rows" :key="i" :class="{ 'row-edited': isRowEdited(i) }">
                <td v-if="mode === 'browse'" class="op-col">
                  <a-button
                    size="small"
                    type="primary"
                    :ghost="!isRowEdited(i)"
                    :loading="savingRow === i"
                    @click="saveRow(i)"
                  >保存</a-button>
                  <a-button v-if="isRowEdited(i)" size="small" @click="revertRow(i)">还原</a-button>
                  <a-popconfirm title="确定删除这一行？" ok-text="删除" ok-type="danger" cancel-text="取消" @confirm="deleteRow(i)">
                    <a-button size="small" danger>删除</a-button>
                  </a-popconfirm>
                </td>
                <td
                  v-for="(cell, j) in row"
                  :key="j"
                  :class="{ 'cell-null': displayCell(i, j) === null && !isCellEdited(i, j), 'cell-modified': isCellEdited(i, j) }"
                  :title="mode === 'browse' ? '双击编辑' : ''"
                  @dblclick="mode === 'browse' && startEdit(i, j)"
                >
                  <input
                    v-if="editingCell && editingCell.r === i && editingCell.c === j"
                    ref="cellInput"
                    class="cell-editor"
                    :value="editBuffer"
                    :placeholder="cell === null ? 'NULL' : ''"
                    @blur="commitEdit()"
                    @keyup.enter="commitEdit()"
                    @keyup.esc="cancelEdit()"
                  />
                  <template v-else>{{ isCellEdited(i, j) ? edits[i][resultInfo.columns[j]] : (cell === null ? 'NULL' : cell) }}</template>
                </td>
              </tr>

              <!-- 新增行 -->
              <tr v-if="addingRow" class="row-adding">
                <td class="op-col">
                  <a-button size="small" type="primary" :loading="savingRow === -1" @click="saveNewRow()">保存</a-button>
                  <a-button size="small" @click="cancelAddRow">取消</a-button>
                </td>
                <td v-for="(c, j) in resultInfo.columns" :key="j">
                  <input
                    class="cell-editor"
                    v-model="addingRow.values[c]"
                    :placeholder="'输入' + c"
                  />
                </td>
              </tr>
            </tbody>
          </table>
        </div>

        <div v-else-if="!errorMsg" class="empty-tip">
          <template v-if="wsStatus === 'connected'">从左侧选择数据表浏览并编辑数据，或在上方输入 SQL 执行</template>
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
      wsStatus: 'disconnected',
      tables: [],
      currentTable: '',
      mode: 'query',
      sql: '',
      running: false,
      savingRow: null, // 正在保存的行号，-1表示新增行
      errorMsg: '',
      resultInfo: null,
      rowids: [],
      pageSize: 100,
      offset: 0,
      edits: {}, // {行号: {列名: 新值}}
      editingCell: null, // {r, c}
      editBuffer: '',
      addingRow: null, // {values: {列名: 值}}
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
      this.clearEdits();
      this.send({ action: 'rows', table: name, limit: this.pageSize, offset: 0 });
    },
    reloadBrowse() {
      this.clearEdits();
      this.send({ action: 'rows', table: this.currentTable, limit: this.pageSize, offset: this.offset });
    },
    loadSchema() {
      if (!this.currentTable) return;
      this.send({ action: 'schema', table: this.currentTable });
    },
    showAuditLog() {
      this.mode = 'query';
      this.sql = 'SELECT * FROM admin_audit_log ORDER BY id DESC LIMIT 100';
      this.runSql();
    },
    runSql() {
      const sql = this.sql.trim();
      if (!sql) return;
      this.running = true;
      this.mode = 'query';
      this.clearEdits();
      this.send({ action: 'query', sql });
    },
    nextPage() {
      this.offset += this.pageSize;
      this.reloadBrowse();
    },
    prevPage() {
      this.offset = Math.max(0, this.offset - this.pageSize);
      this.reloadBrowse();
    },

    // ===== 单元格编辑 =====
    startEdit(r, c) {
      this.editingCell = { r, c };
      const col = this.resultInfo.columns[c];
      const current = this.edits[r] && col in this.edits[r]
        ? this.edits[r][col]
        : this.resultInfo.rows[r][c];
      this.editBuffer = current === null ? '' : String(current);
      this.$nextTick(() => {
        const input = this.$refs.cellInput;
        if (input && input.focus) input.focus();
      });
    },
    commitEdit() {
      if (!this.editingCell) return;
      const { r, c } = this.editingCell;
      const col = this.resultInfo.columns[c];
      if (!this.edits[r]) this.$set(this.edits, r, {});
      this.$set(this.edits[r], col, this.editBuffer);
      this.editingCell = null;
      this.editBuffer = '';
    },
    cancelEdit() {
      this.editingCell = null;
      this.editBuffer = '';
    },
    isCellEdited(r, c) {
      const col = this.resultInfo.columns[c];
      return !!(this.edits[r] && col in this.edits[r]);
    },
    displayCell(r, c) {
      const col = this.resultInfo.columns[c];
      if (this.edits[r] && col in this.edits[r]) return this.edits[r][col];
      return this.resultInfo.rows[r][c];
    },
    isRowEdited(r) {
      return !!this.edits[r] && Object.keys(this.edits[r]).length > 0;
    },
    revertRow(r) {
      this.$delete(this.edits, r);
    },
    clearEdits() {
      this.edits = {};
      this.editingCell = null;
      this.addingRow = null;
    },

    // ===== 行操作：保存/删除/新增 =====
    saveRow(r) {
      const sets = this.edits[r];
      if (!sets || !Object.keys(sets).length) {
        this.$message?.info('该行没有修改需要保存，双击单元格可编辑');
        return;
      }
      this.savingRow = r;
      this.send({ action: 'update', table: this.currentTable, rowid: this.rowids[r], sets });
    },
    deleteRow(r) {
      this.savingRow = r;
      this.send({ action: 'delete', table: this.currentTable, rowid: this.rowids[r] });
    },
    startAddRow() {
      const values = {};
      this.resultInfo.columns.forEach(c => { values[c] = ''; });
      this.addingRow = { values };
    },
    cancelAddRow() {
      this.addingRow = null;
    },
    saveNewRow() {
      if (!this.addingRow) return;
      const values = {};
      Object.entries(this.addingRow.values).forEach(([k, v]) => {
        values[k] = v === '' ? null : v;
      });
      this.savingRow = -1;
      this.send({ action: 'insert', table: this.currentTable, values });
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
        this.savingRow = null;
        return;
      }
      this.errorMsg = '';
      switch (msg.action) {
        case 'tables':
          this.tables = msg.tables || [];
          break;
        case 'rows':
          this.resultInfo = msg;
          this.rowids = msg.rowids || [];
          this.clearEdits();
          break;
        case 'query':
          this.resultInfo = msg;
          this.rowids = [];
          break;
        case 'update':
          this.savingRow = null;
          this.$message?.success(`已更新 ${msg.rows_affected} 行`);
          this.reloadBrowse();
          break;
        case 'delete':
          this.savingRow = null;
          this.$message?.success(`已删除 ${msg.rows_affected} 行`);
          this.reloadBrowse();
          break;
        case 'insert':
          this.savingRow = null;
          this.$message?.success('已新增一行');
          this.reloadBrowse();
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
  padding: 4px 8px;
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
  z-index: 1;
}

.result-grid tbody tr:nth-child(even) { background: #fcfcfc; }
.result-grid tbody tr:hover { background: #f0f5ff; }
.row-edited { background: #fffbe6 !important; }
.row-adding { background: #f6ffed !important; }
.cell-null { color: #c41d7f; font-style: italic; }
.cell-modified { outline: 2px solid #faad14; outline-offset: -2px; }

.op-col {
  position: sticky;
  left: 0;
  background: #fff;
  white-space: nowrap;
  min-width: 130px;
}
.result-grid th.op-col { z-index: 2; }
.op-col .ant-btn { margin-right: 4px; }

.cell-editor {
  width: 100%;
  min-width: 120px;
  border: 1px solid #1677ff;
  border-radius: 3px;
  padding: 2px 6px;
  font-size: 13px;
  font-family: monospace;
  outline: none;
  box-sizing: border-box;
}

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
