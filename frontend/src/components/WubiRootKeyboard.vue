<template>
  <div class="wubi-root-keyboard-container">
    <a-card class="keyboard-card">
      <template #title>
        <span>五笔字根键盘图</span>
      </template>

      <!-- 字根键盘布局 -->
      <div class="keyboard-layout">
        <!-- 区号标题行 -->
        <div class="zone-header-row">
          <div class="zone-label"></div>
          <div class="zone-label zone-1">横区 (1)</div>
          <div class="zone-label zone-2">竖区 (2)</div>
          <div class="zone-label zone-3">撇区 (3)</div>
          <div class="zone-label zone-4">捺区 (4)</div>
          <div class="zone-label zone-5">折区 (5)</div>
        </div>

        <!-- 位号行 -->
        <div class="row" v-for="row in keyboardRows" :key="row.position">
          <div class="position-label">{{ row.position }}</div>
          <div
            v-for="key in row.keys"
            :key="key.char"
            class="key-cell"
            :class="{ 'key-active': activeKey === key.char }"
            @click="selectKey(key.char)"
            @mouseenter="showTooltip(key.char)"
            @mouseleave="hideTooltip"
          >
            <div class="key-letter">{{ key.char.toUpperCase() }}</div>
            <div class="key-formula" v-if="key.formula">{{ key.formula.substring(0, 12) }}...</div>
          </div>
        </div>
      </div>

      <!-- 悬浮提示气泡 -->
      <div v-if="tooltipVisible" class="key-tooltip" :style="tooltipStyle">
        <div class="tooltip-header">{{ tooltipData.key.toUpperCase() }} 键 ({{ tooltipData.code }})</div>
        <div class="tooltip-formula">{{ tooltipData.formula }}</div>
        <div class="tooltip-radicals">{{ tooltipData.radicals }}</div>
      </div>

      <!-- 选中键的详细信息 -->
      <a-card v-if="selectedKeyInfo" class="key-detail-card" :title="`${selectedKeyInfo.key.toUpperCase()} 键详情`">
        <div class="key-detail">
          <p><strong>编码：</strong>{{ selectedKeyInfo.code }}</p>
          <p><strong>口诀：</strong>{{ selectedKeyInfo.formula }}</p>
          <p><strong>主要字根：</strong>{{ selectedKeyInfo.radicals }}</p>
        </div>
      </a-card>
    </a-card>

    <!-- 字根口诀卡片 -->
    <a-card class="formula-card" title="五笔字根口诀">
      <div class="formula-sections">
        <div class="formula-zone zone-1">
          <h3>第一区：横起笔 (G - A)</h3>
          <div class="formula-item" v-for="key in zone1Keys" :key="key.key">
            <span class="formula-key">{{ key.key.toUpperCase() }}</span>
            <span class="formula-formula-text">{{ key.formula }}</span>
            <span class="formula-radicals-list">{{ key.radicals }}</span>
          </div>
        </div>

        <div class="formula-zone zone-2">
          <h3>第二区：竖起笔 (H - M)</h3>
          <div class="formula-item" v-for="key in zone2Keys" :key="key.key">
            <span class="formula-key">{{ key.key.toUpperCase() }}</span>
            <span class="formula-formula-text">{{ key.formula }}</span>
            <span class="formula-radicals-list">{{ key.radicals }}</span>
          </div>
        </div>

        <div class="formula-zone zone-3">
          <h3>第三区：撇起笔 (T - Q)</h3>
          <div class="formula-item" v-for="key in zone3Keys" :key="key.key">
            <span class="formula-key">{{ key.key.toUpperCase() }}</span>
            <span class="formula-formula-text">{{ key.formula }}</span>
            <span class="formula-radicals-list">{{ key.radicals }}</span>
          </div>
        </div>

        <div class="formula-zone zone-4">
          <h3>第四区：捺/点起笔 (Y - P)</h3>
          <div class="formula-item" v-for="key in zone4Keys" :key="key.key">
            <span class="formula-key">{{ key.key.toUpperCase() }}</span>
            <span class="formula-formula-text">{{ key.formula }}</span>
            <span class="formula-radicals-list">{{ key.radicals }}</span>
          </div>
        </div>

        <div class="formula-zone zone-5">
          <h3>第五区：折起笔 (N - X)</h3>
          <div class="formula-item" v-for="key in zone5Keys" :key="key.key">
            <span class="formula-key">{{ key.key.toUpperCase() }}</span>
            <span class="formula-formula-text">{{ key.formula }}</span>
            <span class="formula-radicals-list">{{ key.radicals }}</span>
          </div>
        </div>
      </div>

      <!-- 学习小贴士 -->
      <a-alert class="tips-alert" type="info" show-icon message="学习小贴士">
        <template #description>
          <ul class="tips-list">
            <li><strong>键名字：</strong>每个键位上的口诀第一个字就是"键名字"（如 G 是王、F 是土）。</li>
            <li><strong>成字字根：</strong>字根本身也是一个汉字的（如"石"、"手"、"口"）。</li>
            <li><strong>拆分原则：</strong>书写顺序、取大优先、兼顾直观、能连不交。</li>
          </ul>
        </template>
      </a-alert>
    </a-card>
  </div>
</template>

<script>
export default {
  name: 'WubiRootKeyboard',
  data() {
    return {
      activeKey: null,
      hoverKey: null,
      tooltipVisible: false,
      tooltipStyle: {},
      tooltipData: {},
      keyRadicals: [
        { key: 'g', code: '11', formula: '王旁青头戋（兼）五一', radicals: '王、一、五、戋' },
        { key: 'f', code: '12', formula: '土士二干十寸雨', radicals: '土、士、二、干、十、寸、雨' },
        { key: 'd', code: '13', formula: '大犬三（古）石厂', radicals: '大、犬、三、古、石、厂' },
        { key: 's', code: '14', formula: '木丁西', radicals: '木、丁、西' },
        { key: 'a', code: '15', formula: '工戈草头右框七', radicals: '工、戈、艹、七、廿' },
        { key: 'h', code: '21', formula: '目具上止卜虎皮', radicals: '目、止、卜、虍、上' },
        { key: 'j', code: '22', formula: '日早两竖与虫依', radicals: '日、早、虫、刂、竖' },
        { key: 'k', code: '23', formula: '口与川，字根稀', radicals: '口、川' },
        { key: 'l', code: '24', formula: '田甲方框四车里', radicals: '田、甲、四、车、囗' },
        { key: 'm', code: '25', formula: '山由贝，下框几', radicals: '山、由、贝、几' },
        { key: 't', code: '31', formula: '禾竹一撇双人立', radicals: '禾、竹、丿、彳、攵' },
        { key: 'r', code: '32', formula: '白手看头三二斤', radicals: '白、手、斤、牛' },
        { key: 'e', code: '33', formula: '舟用乃月豕（家）衣', radicals: '舟、用、月、豕、衣' },
        { key: 'w', code: '34', formula: '人八登头单人几', radicals: '人、八、亻' },
        { key: 'q', code: '35', formula: '金勺缺点无尾鱼，犬旁留叉', radicals: '金、饣、勹、儿、夕' },
        { key: 'y', code: '41', formula: '言文方广在四一，高头一捺谁人去', radicals: '言、文、方、广、丶' },
        { key: 'u', code: '42', formula: '立辛两点六门疒（病）', radicals: '立、辛、六、门、疒' },
        { key: 'i', code: '43', formula: '水旁兴头小倒立', radicals: '氵（三点水）、小' },
        { key: 'o', code: '44', formula: '火业头，四点米', radicals: '火、米、灬' },
        { key: 'p', code: '45', formula: '之字军盖建道底，摘礻衤', radicals: '之、宀（宝盖）、冖、礻、衤' },
        { key: 'n', code: '51', formula: '已半巳满不出己，左框折尸心和羽', radicals: '已、己、巳、尸、心、羽' },
        { key: 'b', code: '52', formula: '子耳了也框向上', radicals: '子、耳、了、也、卩' },
        { key: 'v', code: '53', formula: '女刀九臼山朝西', radicals: '女、刀、九、臼' },
        { key: 'c', code: '54', formula: '又巴马，丢矢矣', radicals: '又、巴、马、厶' },
        { key: 'x', code: '55', formula: '慈母无心弓和匕，幼无力', radicals: '幺、母、弓、匕' },
      ],
      keyboardLayout: [
        { position: '1', keys: ['g', 'h', 't', 'y', 'n'] },
        { position: '2', keys: ['f', 'j', 'r', 'u', 'b'] },
        { position: '3', keys: ['d', 'k', 'e', 'i', 'v'] },
        { position: '4', keys: ['s', 'l', 'w', 'o', 'c'] },
        { position: '5', keys: ['a', 'm', 'q', 'p', 'x'] },
      ]
    };
  },
  computed: {
    keyboardRows() {
      return this.keyboardLayout.map(row => ({
        position: row.position,
        keys: row.keys.map(k => {
          const data = this.keyRadicals.find(r => r.key === k);
          return {
            char: k,
            formula: data ? data.formula : '',
            radicals: data ? data.radicals : '',
          };
        })
      }));
    },
    selectedKeyInfo() {
      if (!this.activeKey) return null;
      return this.keyRadicals.find(r => r.key === this.activeKey);
    },
    zone1Keys() {
      return this.keyRadicals.filter(k => ['g', 'f', 'd', 's', 'a'].includes(k.key));
    },
    zone2Keys() {
      return this.keyRadicals.filter(k => ['h', 'j', 'k', 'l', 'm'].includes(k.key));
    },
    zone3Keys() {
      return this.keyRadicals.filter(k => ['t', 'r', 'e', 'w', 'q'].includes(k.key));
    },
    zone4Keys() {
      return this.keyRadicals.filter(k => ['y', 'u', 'i', 'o', 'p'].includes(k.key));
    },
    zone5Keys() {
      return this.keyRadicals.filter(k => ['n', 'b', 'v', 'c', 'x'].includes(k.key));
    }
  },
  methods: {
    showTooltip(key) {
      this.hoverKey = key;
      const data = this.keyRadicals.find(r => r.key === key);
      if (data) {
        this.tooltipData = data;
        this.tooltipVisible = true;
        this.tooltipStyle = {
          left: '50%',
          top: '40%',
        };
      }
    },
    hideTooltip() {
      this.hoverKey = null;
      this.tooltipVisible = false;
    },
    selectKey(key) {
      if (this.activeKey === key) {
        this.activeKey = null;
      } else {
        this.activeKey = key;
      }
    }
  }
};
</script>

<style scoped>
.wubi-root-keyboard-container {
  padding: 20px;
}

.keyboard-card {
  margin-bottom: 24px;
}

.formula-card {
  margin-bottom: 24px;
}

.keyboard-layout {
  padding: 20px;
  background: #f5f5f5;
  border-radius: 8px;
}

.zone-header-row {
  display: grid;
  grid-template-columns: 60px repeat(5, 1fr);
  gap: 8px;
  margin-bottom: 8px;
}

.zone-label {
  text-align: center;
  font-weight: bold;
  padding: 8px;
  border-radius: 4px;
  color: white;
}

.zone-1 { background: #ef4444; }
.zone-2 { background: #f59e0b; }
.zone-3 { background: #22c55e; }
.zone-4 { background: #3b82f6; }
.zone-5 { background: #a855f7; }

.row {
  display: grid;
  grid-template-columns: 60px repeat(5, 1fr);
  gap: 8px;
  margin-bottom: 8px;
}

.position-label {
  display: flex;
  align-items: center;
  justify-content: center;
  font-weight: bold;
  color: #666;
}

.key-cell {
  background: white;
  border: 2px solid #ddd;
  border-radius: 8px;
  padding: 12px 8px;
  text-align: center;
  cursor: pointer;
  transition: all 0.2s ease;
  min-height: 80px;
  display: flex;
  flex-direction: column;
  justify-content: center;
  align-items: center;
  position: relative;
}

.key-cell:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  z-index: 10;
}

.key-cell.key-active {
  border-color: #1890ff;
  background: #e6f7ff;
  box-shadow: 0 0 12px rgba(24, 144, 255, 0.3);
}

.key-letter {
  font-size: 24px;
  font-weight: bold;
  color: #333;
  margin-bottom: 4px;
}

.key-formula {
  font-size: 12px;
  color: #888;
  word-break: break-all;
  line-height: 1.2;
}

.key-tooltip {
  position: fixed;
  left: 50%;
  top: 40%;
  transform: translate(-50%, -50%);
  background: #1e293b;
  color: white;
  padding: 16px 20px;
  border-radius: 8px;
  z-index: 1000;
  min-width: 280px;
  max-width: 400px;
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.3);
  pointer-events: none;
}

.tooltip-header {
  font-size: 18px;
  font-weight: bold;
  color: #fbbf24;
  margin-bottom: 8px;
}

.tooltip-formula {
  font-size: 16px;
  color: #e2e8f0;
  margin-bottom: 8px;
  line-height: 1.4;
}

.tooltip-radicals {
  font-size: 14px;
  color: #94a3b8;
  line-height: 1.4;
}

.key-detail-card {
  margin-top: 16px;
}

.key-detail p {
  margin: 8px 0;
  font-size: 16px;
}

.key-detail strong {
  color: #1890ff;
}

.formula-sections {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
  gap: 20px;
  padding: 16px;
}

.formula-zone {
  background: #fafafa;
  border-radius: 8px;
  padding: 16px;
}

.formula-zone h3 {
  margin: 0 0 12px 0;
  padding: 8px 12px;
  border-radius: 4px;
  color: white;
  font-size: 16px;
}

.zone-1 h3 { background: #ef4444; }
.zone-2 h3 { background: #f59e0b; }
.zone-3 h3 { background: #22c55e; }
.zone-4 h3 { background: #3b82f6; }
.zone-5 h3 { background: #a855f7; }

.formula-item {
  display: flex;
  flex-direction: column;
  padding: 10px 12px;
  margin-bottom: 10px;
  background: white;
  border-radius: 6px;
  border-left: 4px solid;
  gap: 6px;
}

.zone-1 .formula-item { border-left-color: #ef4444; }
.zone-2 .formula-item { border-left-color: #f59e0b; }
.zone-3 .formula-item { border-left-color: #22c55e; }
.zone-4 .formula-item { border-left-color: #3b82f6; }
.zone-5 .formula-item { border-left-color: #a855f7; }

.formula-key {
  font-size: 18px;
  font-weight: bold;
  color: #333;
  text-align: left;
  background: #f0f0f0;
  padding: 2px 10px;
  border-radius: 4px;
  align-self: flex-start;
}

.formula-formula-text {
  font-size: 15px;
  font-weight: bold;
  color: #555;
  line-height: 1.4;
}

.formula-radicals-list {
  font-size: 14px;
  color: #888;
  line-height: 1.4;
}

.tips-alert {
  margin-top: 20px;
}

.tips-list {
  margin: 8px 0 0 0;
  padding-left: 20px;
}

.tips-list li {
  margin-bottom: 6px;
  line-height: 1.5;
}

@media (max-width: 768px) {
  .formula-sections {
    grid-template-columns: 1fr;
  }

  .row {
    grid-template-columns: 40px repeat(5, 1fr);
    gap: 4px;
  }

  .key-cell {
    padding: 8px 4px;
    min-height: 60px;
  }

  .key-letter {
    font-size: 18px;
  }

  .key-formula {
    font-size: 10px;
  }
}
</style>
