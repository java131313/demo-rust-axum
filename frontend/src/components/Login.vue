<template>
  <div class="login-container">
    <a-card title="五笔打字练习系统" class="login-card">
      <a-tabs v-model:activeKey="activeTab">
        <a-tab-pane key="login" tab="登录">
          <a-form :model="loginForm" :rules="loginRules" ref="loginFormRef" @finish="handleLogin">
            <a-form-item name="username" label="用户名">
              <a-input v-model:value="loginForm.username" placeholder="请输入用户名" />
            </a-form-item>
            <a-form-item name="password" label="密码">
              <a-input-password v-model:value="loginForm.password" placeholder="请输入密码" />
            </a-form-item>
            <a-form-item>
              <a-button type="primary" html-type="submit" :loading="loading" style="width: 100%">
                登录
              </a-button>
            </a-form-item>
          </a-form>
        </a-tab-pane>
        <a-tab-pane key="register" tab="注册">
          <a-form :model="registerForm" :rules="registerRules" ref="registerFormRef" @finish="handleRegister">
            <a-form-item name="username" label="用户名">
              <a-input v-model:value="registerForm.username" placeholder="请输入用户名" />
            </a-form-item>
            <a-form-item name="email" label="邮箱">
              <a-input v-model:value="registerForm.email" placeholder="请输入邮箱" />
            </a-form-item>
            <a-form-item name="password" label="密码">
              <a-input-password v-model:value="registerForm.password" placeholder="请输入密码" />
            </a-form-item>
            <a-form-item name="confirmPassword" label="确认密码" :dependencies="['password']">
              <a-input-password v-model:value="registerForm.confirmPassword" placeholder="请再次输入密码" />
            </a-form-item>
            <a-form-item>
              <a-button type="primary" html-type="submit" :loading="loading" style="width: 100%">
                注册
              </a-button>
            </a-form-item>
          </a-form>
        </a-tab-pane>
      </a-tabs>
    </a-card>
  </div>
</template>

<script setup>
import { ref, reactive } from 'vue';
import { message } from 'ant-design-vue';
import { login, register } from '../api';
import { useRouter } from 'vue-router';

const router = useRouter();
const activeTab = ref('login');
const loading = ref(false);

const loginFormRef = ref(null);
const registerFormRef = ref(null);

const loginForm = reactive({
  username: '',
  password: ''
});

const registerForm = reactive({
  username: '',
  email: '',
  password: '',
  confirmPassword: ''
});

const validateConfirmPassword = (rule, value) => {
  if (value !== registerForm.password) {
    return Promise.reject('两次输入的密码不一致');
  }
  return Promise.resolve();
};

const loginRules = {
  username: [{ required: true, message: '请输入用户名', trigger: 'blur' }],
  password: [{ required: true, message: '请输入密码', trigger: 'blur' }]
};

const registerRules = {
  username: [{ required: true, message: '请输入用户名', trigger: 'blur' }],
  email: [
    { required: true, message: '请输入邮箱', trigger: 'blur' },
    { type: 'email', message: '请输入正确的邮箱格式', trigger: 'blur' }
  ],
  password: [{ required: true, message: '请输入密码', trigger: 'blur' }],
  confirmPassword: [
    { required: true, message: '请再次输入密码', trigger: 'blur' },
    { validator: validateConfirmPassword, trigger: 'blur' }
  ]
};

const handleLogin = async () => {
  loading.value = true;
  try {
    const response = await login(loginForm);
    const { access_token, user } = response.data;
    localStorage.setItem('token', access_token);
    localStorage.setItem('user', JSON.stringify(user));
    message.success('登录成功');
    router.push('/');
  } catch (error) {
    message.error(error.response?.data?.message || '登录失败');
  } finally {
    loading.value = false;
  }
};

const handleRegister = async () => {
  loading.value = true;
  try {
    const response = await register(registerForm);
    const { access_token, user } = response.data;
    localStorage.setItem('token', access_token);
    localStorage.setItem('user', JSON.stringify(user));
    message.success('注册成功');
    router.push('/');
  } catch (error) {
    message.error(error.response?.data?.message || '注册失败');
  } finally {
    loading.value = false;
  }
};
</script>

<style scoped>
.login-container {
  display: flex;
  justify-content: center;
  align-items: center;
  min-height: 100vh;
  padding: 24px 16px;
  background: linear-gradient(135deg, #4c6cb3 0%, #5a4a7a 100%);
}

.login-card {
  width: 100%;
  max-width: 480px;
  min-width: min(100%, 320px);
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.2);
}

.login-card :deep(.ant-card-head-title) {
  font-size: 1.35rem;
  font-weight: 700;
}
</style>
