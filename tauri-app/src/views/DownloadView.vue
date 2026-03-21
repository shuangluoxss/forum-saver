<template>
  <div class="download-view">
    <n-card :bordered="false" style="margin-top: -12px;">
      <n-space vertical size="large">
        <n-alert title="使用说明" type="info">
          输入论坛主题帖的完整 URL
          即可开始下载。下载过程将根据您在“配置”页面设置的规则进行。
        </n-alert>

        <n-input-group>
          <n-input v-model:value="url" placeholder="请输入帖子 URL (例如: https://stage1st.com/2b/thread-...)"
            :disabled="loading" @keyup.enter="handleDownload" />
          <n-button type="primary" :loading="loading" @click="handleDownload">
            开始下载
          </n-button>
        </n-input-group>

        <div v-if="loading || logs.length > 0" class="log-container">
          <n-card size="small" title="下载日志" embedded>
            <template #header-extra>
              <n-button quaternary size="tiny" @click="logs = []">清空</n-button>
            </template>

            <!-- 进度条 -->
            <div v-if="progress.current > 0" class="progress-container">
              <n-progress type="line" :percentage="progressPercentage" :format="progressFormat"
                :status="progressStatus">{{
                  progress.current }} / {{ progress.total }}</n-progress>
            </div>

            <div class="log-content" ref="logRef" style="height: 40vh">
              <div v-for="(log, index) in logs" :key="index" :class="['log-line', log.type]">
                <span class="log-time">[{{ log.time }}]</span>
                <span class="log-msg">{{ log.message }}</span>
              </div>
              <div v-if="loading" class="log-line loading">
                <n-spin size="small" />
                正在下载中，请稍候...
              </div>
            </div>
          </n-card>
        </div>
      </n-space>
    </n-card>
  </div>
</template>

<script setup lang="ts">
import { ref, nextTick, computed } from "vue";
import { invoke, Channel } from "@tauri-apps/api/core";

type InfoLevel = "info" | "error" | "success";

// 定义下载信息相关接口
interface TextInfo {
  message: string;
  level: InfoLevel;
  type: 'Text';
}

interface ProgressInfo {
  current: number;
  total: number;
  type: 'Progress';
}

type DownloadInfo =
  | TextInfo
  | ProgressInfo;

// 进度对象
const progress = ref({
  current: 0,
  total: 0
});

const progressPercentage = computed(() => {
  if (progress.value.total > 0) {
    return (progress.value.current / progress.value.total) * 100;
  }
  return 0;
});

const url = ref("");
const loading = ref(false);
const logs = ref<
  { time: string; message: string; type: InfoLevel }[]
>([]);
const logRef = ref<HTMLElement | null>(null);
const message = useMessage();

const progressFormat = (percentage: number) => {
  if (progress.value.total > 0) {
    return `${Math.round(percentage)}% (${Math.round((percentage / 100) * progress.value.total)}/${progress.value.total})`;
  }
  return `${Math.round(percentage)}%`;
};

const progressStatus = computed(() => {
  if (progress.value.current === progress.value.total && progress.value.total > 0) return "success";
  return undefined;
});

const addLog = (msg: string, type: InfoLevel = "info") => {
  const time = new Date()
  const timStr = time.toLocaleTimeString() + `.${time.getMilliseconds()}`.padEnd(4, '0');
  logs.value.push({ time: timStr, message: msg, type });
  nextTick(() => {
    if (logRef.value) {
      logRef.value.scrollTop = logRef.value.scrollHeight;
    }
  });
};

const handleDownload = async () => {
  if (!url.value.trim()) {
    message.warning("请输入 URL");
    return;
  }

  loading.value = true;
  progress.value = { current: 0, total: 0 };
  addLog(`开始下载: ${url.value}`, "info");

  try {
    // 创建通道
    const channel = new Channel<DownloadInfo>();
    // 监听通道消息
    channel.onmessage = (info) => {
      processDownloadInfo(info);
    };

    // 调用后端下载命令
    await invoke("download_thread", { url: url.value.trim(), channel });
    progress.value.current = progress.value.total;
    addLog("下载完成！", "success");
    message.success("下载完成");
  } catch (e) {
    addLog(`下载失败: ${e}`, "error");
    message.error(`下载失败: ${e}`);
  } finally {
    loading.value = false;
  }
};

const processDownloadInfo = (info: DownloadInfo) => {
  switch (info.type) {
    case 'Text':
      const level = info.level;
      addLog(info.message, level);
      break;
    case 'Progress':
      const { current, total } = info;
      if (total > 0) {
        progress.value.total = total;
        progress.value.current = current;
      }
      break;
  }
};
</script>

<style scoped>
.download-view {
  max-width: 900px;
  margin: 0 auto;
}

.log-container {
  margin-top: 12px;
}

.progress-container {
  margin-bottom: 12px;
}

.log-content {
  height: 300px;
  overflow-y: auto;
  font-family: monospace;
  font-size: 13px;
  background-color: #1e1e1e;
  color: #d4d4d4;
  padding: 12px;
  border-radius: 4px;
}

.log-line {
  margin-bottom: 4px;
  line-height: 1.4;
  word-break: break-all;
}

.log-line.error {
  color: #f87171;
}

.log-line.success {
  color: #4ade80;
}

.log-line.info {
  color: #60a5fa;
}

.log-time {
  margin-right: 8px;
  color: #888;
}

.log-line.loading {
  display: flex;
  align-items: center;
  gap: 8px;
  color: #fbbf24;
  margin-top: 8px;
}
</style>
