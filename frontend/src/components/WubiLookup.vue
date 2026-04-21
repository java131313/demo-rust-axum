<template>
  <div class="wubi-lookup-page">
    <a-space direction="vertical" size="large" style="width: 100%">
      <a-input
        v-model:value="inputChar"
        placeholder="输入一个汉字"
        size="large"
        style="max-width: 320px"
        @pressEnter="lookupWubi"
      />
      <a-button type="primary" size="large" @click="lookupWubi">查询</a-button>
      <div v-if="result" class="wubi-lookup-result">
        <p><strong>汉字：</strong>{{ result.character }}</p>
        <p><strong>全码：</strong>{{ result.full_code }}</p>
        <p v-if="result.simple_code"><strong>简码：</strong>{{ result.simple_code }}</p>
      </div>
    </a-space>
  </div>
</template>

<script setup>
import { ref } from 'vue';
import axios from 'axios';

const inputChar = ref('');
const result = ref(null);

const lookupWubi = async () => {
  try {
    const response = await axios.get(`/api/wubi/${inputChar.value}`);
    result.value = response.data;
  } catch (error) {
    console.error('查询失败:', error);
    result.value = null;
  }
};
</script>

<style scoped>
.wubi-lookup-page {
  padding: 8px 4px 24px;
  max-width: 560px;
}

.wubi-lookup-result {
  font-size: 1.15rem;
  line-height: 1.85;
  color: #141414;
}

.wubi-lookup-result p {
  margin: 0.35em 0;
}
</style>