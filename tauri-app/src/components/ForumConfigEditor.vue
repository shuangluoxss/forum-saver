<template>
  <div class="forum-config-editor">
    <n-form-item label="论坛类型" label-placement="left">
      <n-select v-model:value="forumType" :options="forumTypeOptions" />
    </n-form-item>

    <DynamicForm :model="forumData" @update:model="handleUpdateModel" :groups="currentSchema.groups"
      :advanced-mode="advancedMode" :label-placement="labelPlacement" />
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { NFormItem, NSelect } from 'naive-ui';
import type { ForumConfig } from '../types';
import DynamicForm from './DynamicForm.vue';
import { DISCUZ_SCHEMA, NGA_SCHEMA, V2EX_SCHEMA } from '../schemas';

const props = defineProps<{
  modelValue: ForumConfig;
  labelPlacement?: 'top' | 'left';
  advancedMode?: boolean;
}>();

const emit = defineEmits(['update:modelValue']);

const forumTypeOptions = [
  { label: 'Discuz', value: 'Discuz' },
  { label: 'NGA', value: 'NGA' },
  { label: 'V2EX', value: 'V2EX' },
];

const forumType = computed({
  get: () => Object.keys(props.modelValue)[0] as any,
  set: (newType: string) => {
    const currentData = (props.modelValue as any)[Object.keys(props.modelValue)[0]];
    // Initialize default advanced fields if switching to Discuz
    const newData = { ...currentData };
    if (newType === 'Discuz' && !newData.selectors) {
      Object.assign(newData, {
        remove_ads: true,
        remove_user_info: true,
        remove_reply_box: true,
        interval_ms: 1500,
        thread_url_template: 'thread-{tid}-{pn}-1.html',
        selectors: {
          thread_title: 'h1',
          pg_divs: 'div.pg',
          pgbtn: 'div.pgbtn',
          pn_input: "input.px[name='custompage']",
          username: 'strong.vwmy',
          user_info: '#um',
          login_box: 'div.y.pns',
          reply_box: '#f_pst',
          ads: '.wp.a_h',
          charset: "meta[charset='gbk']",
        }
      });
    }
    emit('update:modelValue', { [newType]: newData });
  },
});

const forumData = computed(() => (props.modelValue as any)[forumType.value]);

const handleUpdateModel = (newData: any) => {
  emit('update:modelValue', { [forumType.value]: newData });
};

const currentSchema = computed(() => {
  switch (forumType.value) {
    case 'Discuz': return DISCUZ_SCHEMA;
    case 'NGA': return NGA_SCHEMA;
    case 'V2EX': return V2EX_SCHEMA;
    default: return DISCUZ_SCHEMA;
  }
});
</script>
