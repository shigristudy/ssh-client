<template>
  <div class="tabs-container">
    <n-tabs
      type="card"
      :tabs-padding="6"
      :closable="true"
      v-model:value="activeTab"
      @close="handleClose"
      @add="handleAdd"
    >
      <n-tab-pane
        v-for="tab in tabs"
        :key="tab.id"
        :name="tab.id"
        :tab="tab.name"
      >
        <Terminal :session-id="tab.sessionId" />
      </n-tab-pane>

      <template #suffix>
        <n-button circle tertiary @click="handleAdd">
          <template #icon>
            <n-icon><add-icon /></n-icon>
          </template>
        </n-button>
      </template>
    </n-tabs>
  </div>
</template>

<script lang="ts" setup>
import { ref } from 'vue'
import { NTabs, NTabPane, NButton, NIcon } from 'naive-ui'
import { Add as AddIcon } from '@vicons/ionicons5'
import Terminal from './Terminal.vue'
import { invoke } from '@tauri-apps/api/core'

interface Tab {
  id: string;
  name: string;
  sessionId: string;
  host: string;
}

const props = defineProps<{
  host: string;
  port: number;
  username: string;
  password?: string;
}>()

const tabs = ref<Tab[]>([])
const activeTab = ref<string>('')

async function createSession() {
  const sessionId = await invoke('create_ssh_connection', {
    config: {
      host: props.host,
      port: props.port,
      username: props.username,
      password: props.password
    }
  })
  
  const newTab: Tab = {
    id: crypto.randomUUID(),
    name: `${props.host} (${tabs.value.length + 1})`,
    sessionId: sessionId as string,
    host: props.host
  }
  
  tabs.value.push(newTab)
  activeTab.value = newTab.id
}

function handleAdd() {
  createSession()
}

async function handleClose(name: string) {
  const tab = tabs.value.find(t => t.id === name)
  if (tab) {
    await invoke('close_ssh_connection', { sessionId: tab.sessionId })
    tabs.value = tabs.value.filter(t => t.id !== name)
  }
}

// Create initial session
createSession()
</script>

<style scoped>
.tabs-container {
  height: 100%;
  display: flex;
  flex-direction: column;
}

:deep(.n-tabs-content) {
  flex: 1;
  height: calc(100% - 40px);
  display: flex;
  flex-direction: column;
  overflow: auto; /* Ensure scrolling is available */
}
</style>