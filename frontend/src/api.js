import axios from 'axios';

const api = axios.create({
  // 使用相对路径：开发时由vite代理到后端，生产时由同一Rust服务提供
  baseURL: '/api',
  timeout: 10000,
});

// 添加请求拦截器，添加token
api.interceptors.request.use(
  config => {
    const token = localStorage.getItem('token');
    if (token) {
      config.headers.Authorization = `Bearer ${token}`;
    }
    return config;
  },
  error => {
    return Promise.reject(error);
  }
);

// 用户登录
export const login = (data) => {
  return api.post('/login', data);
};

// 用户注册
export const register = (data) => {
  return api.post('/register', data);
};

// 用户登出
export const logout = () => {
  return api.post('/logout');
};

// 获取英语练习文章
export const getEnglishTexts = () => {
  return api.get('/english-texts');
};

// 获取日语练习文章
export const getJapaneseTexts = () => {
  return api.get('/japanese-texts');
};

// 获取自定义文章（lang: zh/ja/tw，默认zh）
export const getCustomArticles = (lang = 'zh') => {
  return api.get('/custom-articles', { params: { lang } });
};

// 创建自定义文章
export const createCustomArticle = (data) => {
  return api.post('/custom-articles', data);
};

// 更新自定义文章
export const updateCustomArticle = (id, data) => {
  return api.put(`/custom-articles/${id}`, data);
};

// 删除自定义文章
export const deleteCustomArticle = (id) => {
  return api.delete(`/custom-articles/${id}`);
};

// 获取日语键盘布局
export const getJapaneseKeyboards = () => {
  return api.get('/japanese-keyboards');
};

// 获取日语字符
export const getJapaneseCharacters = () => {
  return api.get('/japanese-characters');
};

export default api;
