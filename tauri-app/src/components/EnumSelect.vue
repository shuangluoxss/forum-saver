<template>
  <div class="enum-select">
    <n-form-item :label="label">
      <n-select
        :value="currentTag"
        :options="options"
        @update:value="handleTagChange"
        placeholder="请选择类型"
      />
    </n-form-item>
    <div v-if="currentTag" class="enum-content">
      <slot :name="currentTag"></slot>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';

const props = defineProps<{
  label: string;
  modelValue: any;
  options: { label: string; value: string }[];
}>();

const emit = defineEmits(['update:modelValue']);

const currentTag = computed(() => {
  if (typeof props.modelValue === 'string') {
    return props.modelValue;
  }
  if (typeof props.modelValue === 'object' && props.modelValue !== null) {
    return Object.keys(props.modelValue)[0];
  }
  return null;
});

const handleTagChange = (newTag: string) => {
  // If the new tag is a simple string enum variant
  if (newTag === 'Guest') {
    emit('update:modelValue', 'Guest');
  } else {
    // For object variants, we need to initialize with a default value
    // This is a simplified approach; in a real app, you might want more specific defaults
    emit('update:modelValue', { [newTag]: {} });
  }
};
</script>

<style scoped>
.enum-content {
  margin-top: 12px;
  padding-left: 16px;
  border-left: 2px solid #eee;
}
</style>
