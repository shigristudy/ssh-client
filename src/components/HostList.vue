<template>
  <n-list>
    <n-list-item v-for="host in hostsStore.hosts" :key="host.id">
      <n-thing :title="host.name" :description="`${host.username}@${host.host}:${host.port}`">
        <template #action>
          <n-space>
            <n-button @click="handleConnect(host)">
              Connect
            </n-button>
            <n-button @click="handleDelete(host.id)">
              Delete
            </n-button>
          </n-space>
        </template>
      </n-thing>
    </n-list-item>
  </n-list>
</template>

<script setup lang="ts">
import { useRouter } from 'vue-router'
import { 
  NList, 
  NListItem, 
  NThing, 
  NButton, 
  NSpace,
  useMessage 
} from 'naive-ui'
import { useHostsStore } from '../stores/hosts'
import { useTerminalsStore } from '../stores/terminals'

const router = useRouter()
const hostsStore = useHostsStore()
const terminalsStore = useTerminalsStore()
const message = useMessage()

interface Host {
  id: string;
  name: string;
  host: string;
  port: number;
  username: string;
  password?: string;
}

async function handleConnect(host: Host) {
  try {
    const terminalId = await terminalsStore.createTerminal({
      host: host.host,
      port: host.port,
      username: host.username,
      password: hostsStore.getHostPassword(host)
    })
    
    router.push({ name: 'terminal', params: { id: terminalId } })
  } catch (error) {
    message.error('Failed to connect: ' + error)
  }
}

async function handleDelete(id: string) {
  try {
    await hostsStore.deleteHost(id)
    message.success('Host deleted successfully')
  } catch (error) {
    message.error('Failed to delete host: ' + error)
  }
}
</script>