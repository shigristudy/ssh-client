
<template>
  <n-drawer
    v-model:show="logsStore.isVisible"
    :width="500"
    placement="right"
  >
    <n-drawer-content title="Application Logs">
      <template #header-extra>
        <n-space>
          <n-button size="small" @click="handleClear">
            Clear
          </n-button>
          <n-switch v-model:value="autoScroll" size="small">
            <template #checked>Auto-scroll</template>
            <template #unchecked>Auto-scroll</template>
          </n-switch>
        </n-space>
      </template>
      
      <div class="log-container" ref="logContainerRef">
        <div 
          v-for="(log, index) in logsStore.logs" 
          :key="index"
          class="log-entry"
          :class="log.level"
        >
          <span class="timestamp">{{ formatTimestamp(log.timestamp) }}</span>
          <span class="source">[{{ log.source }}]</span>
          <span class="level">[{{ log.level }}]</span>
          <span class="message">{{ log.message }}</span>
        </div>
      </div>
    </n-drawer-content>
  </n-drawer>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import { NDrawer, NDrawerContent, NSpace, NButton, NSwitch } from 'naive-ui'
import { useLogsStore } from '../stores/logs'

const logsStore = useLogsStore()
const logContainerRef = ref<HTMLElement | null>(null)
const autoScroll = ref(true)

function handleClear() {
  logsStore.clearLogs()
}

function formatTimestamp(timestamp: number): string {
  return new Date(timestamp).toLocaleTimeString()
}

// Auto-scroll to bottom when new logs are added
watch(() => logsStore.logs.length, () => {
  if (autoScroll.value && logContainerRef.value) {
    setTimeout(() => {
      logContainerRef.value?.scrollTo({
        top: logContainerRef.value.scrollHeight,
        behavior: 'smooth'
      })
    }, 0)
  }
})
</script>

<style scoped>
.log-container {
  height: calc(100vh - 150px);
  overflow-y: auto;
  font-family: 'Menlo', monospace;
  font-size: 12px;
  line-height: 1.4;
  padding: 8px;
  background: #1e1e1e;
  color: #d4d4d4;
  border-radius: 4px;
}

.log-entry {
  white-space: pre-wrap;
  word-wrap: break-word;
  padding: 2px 0;
}

.timestamp {
  color: #666;
  margin-right: 8px;
}

.source {
  color: #569cd6;
  margin-right: 8px;
}

.level {
  margin-right: 8px;
}

.error {
  color: #f14c4c;
}

.warn {
  color: #cca700;
}

.info {
  color: #3b8eea;
}

.debug {
  color: #888;
}
</style>