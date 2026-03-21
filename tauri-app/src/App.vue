<template>
  <n-config-provider :theme="null">
    <n-message-provider>
      <n-tabs default-value="download" type="card" tab-style="font-size: 18px;">
        <n-tab-pane name="download" tab="下载" display-directive="show">
          <DownloadView />
        </n-tab-pane>
        <n-tab-pane name="config" tab="配置" display-directive="show">
          <ConfigView />
        </n-tab-pane>
      </n-tabs>
    </n-message-provider>
  </n-config-provider>
</template>

<script setup lang="ts">
import { NConfigProvider, NMessageProvider, NTabs, NTabPane } from "naive-ui";
import ConfigView from "./views/ConfigView.vue";
import DownloadView from "./views/DownloadView.vue";
import { checkForUpdates } from "./update";

onMounted(async () => {
  await checkForUpdates();
});

</script>

<style>
#app {
  font-family:
    v-sans,
    system-ui,
    -apple-system,
    BlinkMacSystemFont,
    "Segoe UI",
    sans-serif;
}
</style>
