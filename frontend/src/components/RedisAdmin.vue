<template>
  <div class="redis-admin">
    <!-- 工具栏 -->
    <div class="rd-toolbar">
      <span class="ws-dot" :class="wsStatus"></span>
      <span class="ws-text">{{ wsStatusText }}</span>
      <a-button size="small" @click="connect">重新连接</a-button>
      <a-input
        v-model:value="pattern"
        size="small"
        class="pattern-input"
        placeholder="key匹配模式，如 wubi:cache:*"
        @keyup.enter="scanFirst"
      />
      <a-button size="small" type="primary" :loading="scanning" @click="scanFirst">查询</a-button>
      <a-popconfirm
        title="按当前模式清除匹配的缓存key？应用会自动重建缓存。"
        ok-text="清除"
        ok-type="danger"
        cancel-text="取消"
        @confirm="clearCache"
      >
        <a-button size="small" danger>清除缓存</a-button>
      </a-popconfirm>
      <span class="rd-hint">Redis 缓存管理（SCAN 遍历，托管实例兼容）</span>
    </div>

    <a-alert v-if="errorMsg" type="error" :message="errorMsg" closable class="error-bar" @close="errorMsg = ''" />

    <!-- key 列表 -->
    <div class="keys-grid" v-if="keys.length">
      <table>
        <thead>
          <tr>
            <th>Key</th>
            <th class="type-col">类型</th>
            <th class="ttl-col">TTL(秒)</th>
            <th class="op-col">操作</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="k in keys" :key="k.key">
            <td class="key-cell" :title="k.key">{{ k.key }}</td>
            <td class="type-col"><span class="type-badge" :class="'t-' + k.type">{{ k.type }}</span></td>
            <td class="ttl-col">{{ k.ttl === -1 ? '永不过期' : (k.ttl === -2 ? '不存在' : k.ttl) }}</td>
            <td class="op-col">
              <a-button size="small" type="primary" @click="openEditor(k)">查看/编辑</a-button>
              <a-popconfirm title="删除该key？" ok-text="删除" ok-type="danger" cancel-text="取消" @confirm="delKey(k.key)">
                <a-button size="small" danger>删除</a-button>
              </a-popconfirm>
            </td>
          </tr>
        </tbody>
      </table>
      <div class="load-more">
        <a-button size="small" :loading="scanning" v-if="nextCursor !== 0" @click="scanMore">加载更多</a-button>
        <span v-else class="no-more">共 {{ keys.length }} 个key · 已加载全部</span>
      </div>
    </div>
    <div v-else class="empty-tip">
      <template v-if="wsStatus === 'connected'">输入key模式后点"查询"，默认列出应用缓存键</template>
      <template v-else>WebSocket 未连接，点击"重新连接"</template>
    </div>

    <!-- 编辑抽屉 -->
    <a-drawer
      v-model:open="editorOpen"
      :title="editKey || 'key'"
      width="640"
      :destroy-on-close="true"
    >
      <div v-if="keyData" class="editor-body">
        <!-- string -->
        <template v-if="keyData.type === 'string'">
          <a-textarea v-model:value="stringValue" :auto-size="{ minRows: 4, maxRows: 16 }" class="mono" />
          <div class="editor-actions">
            <a-button type="primary" :loading="saving" @click="saveString">保存</a-button>
          </div>
        </template>

        <!-- hash -->
        <template v-else-if="keyData.type === 'hash'">
          <table class="kv-table">
            <thead><tr><th>字段</th><th>值</th><th class="op-col">操作</th></tr></thead>
            <tbody>
              <tr v-for="(p, i) in keyData.entries" :key="i">
                <td class="mono">{{ p[0] }}</td>
                <td>
                  <div class="kv-edit">
                    <a-input v-model:value="hashEdits[i]" size="small" class="mono" />
                    <a-button size="small" type="primary" :loading="saving" @click="saveHashField(p[0], i)">保存</a-button>
                    <a-popconfirm title="删除该字段？" ok-text="删除" ok-type="danger" cancel-text="取消" @confirm="delHashField(p[0])">
                      <a-button size="small" danger>删除</a-button>
                    </a-popconfirm>
                  </div>
                </td>
              </tr>
            </tbody>
          </table>
          <div class="add-row">
            <a-input v-model:value="newHashField" size="small" placeholder="新字段名" style="width: 160px" />
            <a-input v-model:value="newHashValue" size="small" placeholder="值" style="flex: 1" />
            <a-button size="small" type="primary" :loading="saving" @click="addHashField">添加字段</a-button>
          </div>
        </template>

        <!-- list / set / zset -->
        <template v-else-if="['list', 'set', 'zset'].includes(keyData.type)">
          <table class="kv-table">
            <thead>
              <tr>
                <th style="width: 60px">#</th>
                <th v-if="keyData.type === 'zset'">分数</th>
                <th>成员</th>
                <th class="op-col">操作</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="(item, i) in keyData.entries" :key="i">
                <template v-if="keyData.type === 'zset'">
                  <td>{{ i + 1 }}</td>
                  <td class="mono">{{ item[1] }}</td>
                  <td class="mono">{{ item[0] }}</td>
                  <td class="op-col">
                    <a-popconfirm title="删除该成员？" ok-text="删除" ok-type="danger" cancel-text="取消" @confirm="delElement(item[0])">
                      <a-button size="small" danger>删除</a-button>
                    </a-popconfirm>
                  </td>
                </template>
                <template v-else>
                  <td>{{ i + 1 }}</td>
                  <td class="mono">{{ item }}</td>
                  <td class="op-col">
                    <a-popconfirm title="删除该元素？" ok-text="删除" ok-type="danger" cancel-text="取消" @confirm="delElement(item)">
                      <a-button size="small" danger>删除</a-button>
                    </a-popconfirm>
                  </td>
                </template>
              </tr>
            </tbody>
          </table>
          <div class="add-row">
            <a-input
              v-model:value="newElement"
              size="small"
              :placeholder="keyData.type === 'zset' ? '格式: 分数 成员' : '新元素值'"
              style="flex: 1"
            />
            <a-button size="small" type="primary" :loading="saving" @click="addElement">添加</a-button>
          </div>
        </template>

        <template v-else>
          <a-alert type="warning" message="不支持的类型或key不存在" />
        </template>
      </div>
      <div v-else class="empty-tip">加载中…</div>
    </a-drawer>
  </div>
</template>

<script>
export default {
  name: 'RedisAdmin',
  data() {
    return {
      ws: null,
      wsStatus: 'disconnected',
      pattern: 'wubi:cache:*',
      scanning: false,
      keys: [],
      nextCursor: 0,
      errorMsg: '',
      editorOpen: false,
      editKey: '',
      keyData: null,
      hashEdits: {},
      stringValue: '',
      newHashField: '',
      newHashValue: '',
      newElement: '',
      saving: false,
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
        this.errorMsg = '未登录，请先登录后再使用Redis管理';
        this.wsStatus = 'error';
        return;
      }
      this.wsStatus = 'connecting';
      const proto = location.protocol === 'https:' ? 'wss' : 'ws';
      const ws = new WebSocket(`${proto}://${location.host}/ws/redisadmin?token=${encodeURIComponent(token)}`);
      this.ws = ws;
      ws.onopen = () => {
        this.wsStatus = 'connected';
        this.errorMsg = '';
        if (!this.keys.length) this.scanFirst();
      };
      ws.onmessage = (e) => this.onMessage(e.data);
      ws.onclose = () => {
        if (this.ws === ws) this.wsStatus = 'disconnected';
      };
      ws.onerror = () => {
        if (this.ws === ws) this.wsStatus = 'error';
      };
    },
    send(obj) {
      if (!this.ws || this.ws.readyState !== WebSocket.OPEN) {
        this.errorMsg = '连接未就绪';
        return false;
      }
      this.ws.send(JSON.stringify(obj));
      return true;
    },
    scanFirst() {
      this.keys = [];
      this.nextCursor = 0;
      this.scanning = true;
      this.send({ action: 'scan', pattern: this.pattern || '*', cursor: 0 });
    },
    scanMore() {
      this.scanning = true;
      this.send({ action: 'scan', pattern: this.pattern || '*', cursor: this.nextCursor });
    },
    clearCache() {
      this.send({ action: 'clear', pattern: this.pattern || 'wubi:cache:*' });
    },
    delKey(key) {
      this.send({ action: 'del', key });
    },
    openEditor(k) {
      this.editKey = k.key;
      this.editorOpen = true;
      this.keyData = null;
      this.hashEdits = {};
      this.stringValue = '';
      this.newElement = '';
      this.newHashField = '';
      this.newHashValue = '';
      this.send({ action: 'get', key: k.key });
    },
    saveString() {
      this.saving = true;
      this.send({ action: 'set', key: this.editKey, value: this.stringValue });
    },
    saveHashField(field, idx) {
      this.saving = true;
      this.send({ action: 'hset', key: this.editKey, field, value: this.hashEdits[idx] ?? '' });
    },
    delHashField(field) {
      this.saving = true;
      this.send({ action: 'hdel', key: this.editKey, field });
    },
    addHashField() {
      if (!this.newHashField) return;
      this.saving = true;
      this.send({ action: 'hset', key: this.editKey, field: this.newHashField, value: this.newHashValue });
    },
    addElement() {
      if (!this.newElement) return;
      this.saving = true;
      this.send({ action: 'add_elem', key: this.editKey, kind: this.keyData.type, value: this.newElement });
    },
    delElement(value) {
      this.saving = true;
      this.send({ action: 'del_elem', key: this.editKey, kind: this.keyData.type, value });
    },
    reloadEditor() {
      if (this.editorOpen && this.editKey) {
        this.send({ action: 'get', key: this.editKey });
      }
    },
    onMessage(raw) {
      this.scanning = false;
      this.saving = false;
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
        case 'scan':
          this.keys = this.keys.concat(msg.keys || []);
          this.nextCursor = msg.next_cursor;
          break;
        case 'get':
          if (msg.key === this.editKey) {
            this.keyData = msg.data;
            this.stringValue = msg.data.type === 'string' ? (msg.data.value ?? '') : '';
            if (msg.data.type === 'hash') {
              const edits = {};
              (msg.data.entries || []).forEach((p, i) => { edits[i] = p[1]; });
              this.hashEdits = edits;
            }
          }
          break;
        case 'set':
          this.$message?.success('已保存');
          this.reloadEditor();
          break;
        case 'hset':
          this.$message?.success('字段已保存');
          this.newHashField = '';
          this.newHashValue = '';
          this.reloadEditor();
          break;
        case 'hdel':
          this.$message?.success('字段已删除');
          this.reloadEditor();
          break;
        case 'add_elem':
          this.$message?.success('元素已添加');
          this.newElement = '';
          this.reloadEditor();
          break;
        case 'del_elem':
          this.$message?.success('元素已删除');
          this.reloadEditor();
          break;
        case 'del':
          this.$message?.success('key已删除');
          this.editorOpen = false;
          this.scanFirst();
          break;
        case 'clear':
          this.$message?.success(`已清除 ${msg.deleted} 个key${msg.finished ? '' : '（数量较大，仅处理了一部分，可再点一次）'}`);
          this.scanFirst();
          break;
      }
    },
  },
};
</script>

<style scoped>
.redis-admin {
  padding: 12px;
  background: #f5f5f5;
  border-radius: 8px;
  min-height: 480px;
  display: flex;
  flex-direction: column;
}

.rd-toolbar {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 10px;
  padding: 8px 12px;
  background: #fff;
  border-radius: 6px;
  flex-wrap: wrap;
}

.ws-dot { width: 10px; height: 10px; border-radius: 50%; background: #bbb; }
.ws-dot.connected { background: #52c41a; }
.ws-dot.connecting { background: #faad14; }
.ws-dot.error { background: #f5222d; }
.ws-text { font-size: 13px; color: #666; min-width: 48px; }
.pattern-input { width: 220px; font-family: monospace; }
.rd-hint { color: #999; font-size: 12px; margin-left: auto; }

.error-bar { margin-bottom: 10px; }

.keys-grid {
  background: #fff;
  border-radius: 6px;
  overflow: auto;
  flex: 1;
}

.keys-grid table { border-collapse: collapse; width: 100%; font-size: 13px; }
.keys-grid th, .keys-grid td {
  border: 1px solid #f0f0f0;
  padding: 6px 10px;
  text-align: left;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.keys-grid th { background: #fafafa; position: sticky; top: 0; }
.key-cell { font-family: monospace; max-width: 420px; }
.type-col { width: 90px; }
.ttl-col { width: 110px; }
.op-col { width: 170px; white-space: nowrap; }
.op-col .ant-btn { margin-right: 4px; }

.type-badge {
  display: inline-block;
  padding: 1px 8px;
  border-radius: 10px;
  font-size: 12px;
  background: #f0f0f0;
}
.t-string { background: #e6f4ff; color: #1677ff; }
.t-hash { background: #f6ffed; color: #52c41a; }
.t-list { background: #fff7e6; color: #fa8c16; }
.t-set { background: #fff0f6; color: #c41d7f; }
.t-zset { background: #f9f0ff; color: #722ed1; }

.load-more { padding: 10px; text-align: center; }
.no-more { color: #bbb; font-size: 12px; }

.empty-tip {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  color: #bbb;
  font-size: 13px;
  background: #fff;
  border-radius: 6px;
}

.editor-body { display: flex; flex-direction: column; gap: 12px; }
.mono { font-family: monospace; font-size: 13px; }
.editor-actions { display: flex; justify-content: flex-end; gap: 8px; }

.kv-table { border-collapse: collapse; width: 100%; font-size: 13px; }
.kv-table th, .kv-table td { border: 1px solid #f0f0f0; padding: 5px 8px; text-align: left; }
.kv-table th { background: #fafafa; }
.kv-edit { display: flex; gap: 6px; }

.add-row { display: flex; gap: 8px; margin-top: 8px; }

@media (max-width: 768px) {
  .pattern-input { width: 100%; }
}
</style>
