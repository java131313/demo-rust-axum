<template>
  <div>
    <a-input v-model:value="inputChar" placeholder="输入汉字" @pressEnter="lookupWubi" />
    <a-button type="primary" @click="lookupWubi">查询</a-button>
    <div v-if="result">
      <p>汉字: {{ result.character }}</p>
      <p>五笔编码: {{ result.wubi_code }}</p>
    </div>
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