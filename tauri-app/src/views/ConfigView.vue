<template>
  <n-layout has-sider position="absolute" style="top: 50px; bottom: 0">
    <n-layout-sider bordered collapse-mode="width" :collapsed-width="0" :width="180" show-trigger>
      <div style="display: flex; flex-direction: column; height: 100%">
        <n-list hoverable clickable>
          <n-list-item @click="activeForumIndex = -1" :class="{ 'active-item': activeForumIndex === -1 }"
            style="border-bottom: 1px solid #eee">
            <n-text strong>通用设置</n-text>
          </n-list-item>
          <n-list-item style="border-bottom: 1px solid #eee">
            <n-text strong>论坛列表</n-text>
          </n-list-item>
        </n-list>

        <n-layout :native-scrollbar="false" content-style="padding-left: 16px; flex: 1; min-height: 0;">
          <n-list hoverable clickable>
            <n-list-item v-for="(forum, index) in config.forums" :key="index"
              :class="{ 'active-item': activeForumIndex === index }" @click="activeForumIndex = index"
              style="height: 40px; padding: 8px 16px">
              <n-text>{{ getForumDisplayName(forum) }}</n-text>
              <template #suffix>
                <n-button quaternary circle type="error" @click.stop="removeForum(index)" size="small">
                  <n-icon size="18">
                    <TrashOutline />
                  </n-icon>
                </n-button>
              </template>
            </n-list-item>
          </n-list>
        </n-layout>

        <n-layout-footer style="padding: 12px;">
          <n-button dashed block type="primary" @click="addForum">
            + 添加新论坛
          </n-button>
        </n-layout-footer>
      </div>
    </n-layout-sider>

    <n-layout-content content-style="padding: 24px; height: 100%; display: flex; flex-direction: column;">
      <template v-if="activeForumIndex === -1">
        <n-flex justify="space-between" align="center" style="margin-top: -18px; margin-bottom: 12px; flex-shrink: 0;">
          <n-h2 style="margin: 0">全局配置</n-h2>
          <n-flex align="center">
            <n-text strong>高级模式</n-text>
            <n-switch v-model:value="advancedMode" />
          </n-flex>
        </n-flex>
        <n-scrollbar style="flex: 1; min-height: 0;">
          <DynamicForm v-model:model="config" :groups="GLOBAL_SCHEMA.groups" :advanced-mode="advancedMode"
            @pick-dir="handlePickDir" />
        </n-scrollbar>
      </template>

      <template v-else-if="activeForumIndex >= 0">
        <n-flex justify="space-between" align="center" style="margin-top: -18px; margin-bottom: 12px; flex-shrink: 0;">
          <n-h2 style="margin: 0">论坛配置:
            {{ getForumDisplayName(config.forums[activeForumIndex]) }}</n-h2>
          <n-flex align="center">
            <n-text strong>高级模式</n-text>
            <n-switch v-model:value="advancedMode" />
          </n-flex>
        </n-flex>
        <n-scrollbar style="flex: 1; min-height: 0;">
          <ForumConfigEditor v-model="config.forums[activeForumIndex]" :advanced-mode="advancedMode" />
        </n-scrollbar>
      </template>

      <n-divider style="flex-shrink: 0; margin: 24px 0 12px 0" />
      <n-flex justify="end" style="flex-shrink: 0; margin-bottom: -6px;">
        <n-button type="primary" size="large" @click="handleSave">保存配置</n-button>
      </n-flex>
    </n-layout-content>
  </n-layout>
</template>

<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import type { DownloaderConfig, ForumConfig } from "../types";
import ForumConfigEditor from "../components/ForumConfigEditor.vue";
import DynamicForm from "../components/DynamicForm.vue";
import { GLOBAL_SCHEMA } from "../schemas";
import { TrashOutline } from "@vicons/ionicons5";

const message = useMessage();
const activeForumIndex = ref<number>(-1);
const advancedMode = ref(false);

const config = ref<DownloaderConfig>({
  forums: [],
  downloadable_attrs: [
    "href",
    "src",
    "data-src",
    "file",
    "zoomfile",
    "poster",
    "style",
  ],
  downloadable_extensions: [
    "png",
    "jpg",
    "jpeg",
    "gif",
    "svg",
    "webp",
    "tiff",
    "bmp",
    "js",
    "mjs",
    "css",
    "scss",
    "less",
    "woff",
    "woff2",
    "ttf",
    "eot",
    "otf",
    "mp4",
    "webm",
    "ogg",
    "mp3",
    "wav",
    "aac",
    "flac",
  ],
  max_path_length: 240,
  path_hash_length: 16,
  max_depth: 3,
  store_dir: "./data",
  semaphore_count: 8,
  language: "zh",
  strategy: "resume-latest",
});

const loadConfig = async () => {
  try {
    const res = await invoke<DownloaderConfig>("load_config");
    config.value = res;
    message.success("配置已加载: ~/.config/forum-saver/config.toml");
  } catch (e) {
    message.error(`加载配置失败: ${e}`);
  }
};

onMounted(() => {
  loadConfig();
});

const getForumDisplayName = (forum: ForumConfig) => {
  const type = Object.keys(forum)[0];
  const data = (forum as any)[type];
  return data.name || `未命名 ${type}`;
};

const addForum = () => {
  const newForum: ForumConfig = {
    Discuz: {
      name: "",
      base_url: "",
      auth_method: "Guest",
    },
  };
  config.value.forums.push(newForum);
  activeForumIndex.value = config.value.forums.length - 1;
};

const removeForum = (index: number) => {
  config.value.forums.splice(index, 1);
  if (activeForumIndex.value >= config.value.forums.length) {
    activeForumIndex.value = config.value.forums.length - 1;
  }
};

const handlePickDir = async () => {
  try {
    const selected = await open({
      directory: true,
      multiple: false,
      defaultPath: config.value.store_dir,
    });
    if (selected && typeof selected === "string") {
      config.value.store_dir = selected;
    }
  } catch (e) {
    message.error(`打开目录选择器失败: ${e}`);
  }
};

const handleSave = async () => {
  try {
    await invoke("save_config", { config: config.value });
    message.success("配置已保存至 ~/.config/forum-saver/config.toml");
  } catch (e) {
    message.error(`保存失败: ${e}`);
  }
};
</script>

<style scoped>
.active-item {
  background-color: #e6f7ff !important;
  border-right: 3px solid #1890ff;
}
</style>
