<template>
  <div class="auth-method-editor" style="margin-top: -25px; ">
    <n-form-item>
      <n-select v-model:value="currentTag" :options="authOptions" />
    </n-form-item>

    <div v-if="currentTag === 'Guest'" class="auth-fields" />
    <DynamicForm v-else-if="currentTag" :model="modelValue" @update:model="handleUpdate"
      :groups="AUTH_METHOD_SCHEMAS[currentTag]" />
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { NFormItem, NSelect } from 'naive-ui';
import type { AuthMethod } from '../types';
import { AUTH_METHOD_SCHEMAS } from '../schemas';

// DynamicForm is now registered globally in main.ts to avoid circular dependency

const props = defineProps<{
  modelValue: AuthMethod;
}>();

const emit = defineEmits(['update:modelValue']);

const authOptions = [
  { label: '游客 (Guest)', value: 'Guest' },
  { label: 'Cookie 字符串 (CookieString)', value: 'CookieString' },
  { label: '从浏览器读取 (CookieFromBrowser)', value: 'CookieFromBrowser' },
  { label: '用户名密码 (UsernamePassword)', value: 'UsernamePassword' },
];

const currentTag = computed({
  get: () => {
    if (typeof props.modelValue === 'string') return props.modelValue;
    return Object.keys(props.modelValue)[0] as any;
  },
  set: (newTag: string) => {
    let newValue: AuthMethod;
    switch (newTag) {
      case 'Guest':
        newValue = 'Guest';
        break;
      case 'CookieString':
        newValue = { CookieString: '' };
        break;
      case 'CookieFromBrowser':
        newValue = { CookieFromBrowser: 'Chrome' };
        break;
      case 'UsernamePassword':
        newValue = { UsernamePassword: { username: '', password: '' } };
        break;
      default:
        newValue = 'Guest';
    }
    emit('update:modelValue', newValue);
  },
});

const handleUpdate = (val: any) => {
  emit('update:modelValue', val);
};
</script>

<style scoped>
.auth-method-editor {
  width: 100%;
}

.auth-fields {
  margin-top: 8px;
  padding: 16px;
  /* background-color: #fafafc; */
  border-radius: 4px;
  width: 100%;
}
</style>
