import axios from 'axios';

const api = axios.create({
  baseURL: 'http://localhost:3000/api',
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

export default api;
