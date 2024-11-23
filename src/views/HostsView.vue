<template>
  <div class="hosts-view">
    <n-space vertical :size="24">
      <n-card>
        <n-space justify="space-between">
          <h2>SSH Hosts</h2>
          <n-button type="primary" @click="showModal = true">
            Add Host
          </n-button>
        </n-space>
      </n-card>

      <n-card title="Host Groups">
        <n-tabs type="line" v-model:value="activeGroup">
          <n-tab-pane name="all" tab="All Hosts" />
          <n-tab-pane v-for="group in availableGroups" :key="group" :name="group" :tab="group" />
        </n-tabs>

        <n-data-table
          :columns="columns"
          :data="filteredHosts"
          :pagination="{ pageSize: 10 }"
        />
      </n-card>
    </n-space>

    <!-- Add/Edit Host Modal -->
    <n-modal v-model:show="showModal">
      <n-card title="Add Host" style="width: 600px">
        <n-form ref="formRef" :model="formModel" :rules="rules">
          <n-form-item label="Name" path="name">
            <n-input v-model:value="formModel.name" placeholder="My Server" />
          </n-form-item>
          <n-form-item label="Host" path="host">
            <n-input v-model:value="formModel.host" placeholder="example.com" />
          </n-form-item>
          <n-form-item label="Port" path="port">
            <n-input-number v-model:value="formModel.port" :min="1" :max="65535" />
          </n-form-item>
          <n-form-item label="Username" path="username">
            <n-input v-model:value="formModel.username" placeholder="root" />
          </n-form-item>
          <n-form-item label="Password" path="password">
            <n-input
              v-model:value="formModel.password"
              type="password"
              show-password-on="click"
            />
          </n-form-item>
          <n-form-item label="Group" path="group">
            <n-input v-model:value="formModel.group" placeholder="Production" />
          </n-form-item>
        </n-form>
        <template #footer>
          <n-space justify="end">
            <n-button @click="showModal = false">Cancel</n-button>
            <n-button type="primary" :loading="saving" @click="handleSubmit">Save</n-button>
          </n-space>
        </template>
      </n-card>
    </n-modal>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, h } from 'vue'
import { useHostsStore } from '../stores/hosts'
import { useTerminalsStore } from '../stores/terminals'
import { useRouter } from 'vue-router'
import type { Host } from '../types/ssh'
import { NButton, NSpace, NCard, useMessage, FormInst } from 'naive-ui'

const router = useRouter()
const hostsStore = useHostsStore()
const terminalsStore = useTerminalsStore()
const message = useMessage()

const showModal = ref(false)
const activeGroup = ref('all')
const formRef = ref<FormInst | null>(null)
const saving = ref(false)
const connecting = ref(false)
const activeHost = ref<Host | null>(null) // Define activeHost

interface FormModel {
  name: string
  host: string
  port: number
  username: string
  password: string
  group: string
}

const formModel = ref<FormModel>({
  name: '',
  host: '',
  port: 22,
  username: '',
  password: '',
  group: ''
})

const rules = {
  name: { required: true, message: 'Please enter a name' },
  host: { required: true, message: 'Please enter a host' },
  username: { required: true, message: 'Please enter a username' }
}

const columns = [
  { title: 'Name', key: 'name' },
  { title: 'Host', key: 'host' },
  { title: 'Port', key: 'port' },
  { title: 'Username', key: 'username' },
  { title: 'Group', key: 'group' },
  {
    title: 'Actions',
    key: 'actions',
    render: (row: Host) => {
      return h(NSpace, {}, {
        default: () => [
          h(NButton, { 
            size: 'small',
            type: 'primary',
            loading: connecting.value && activeHost.value?.id === row.id,
            onClick: () => handleConnect(row) 
          }, { default: () => 'Connect' }),
          h(NButton, { 
            size: 'small',
            type: 'error',
            onClick: () => handleDelete(row) 
          }, { default: () => 'Delete' })
        ]
      })
    }
  }
]

const filteredHosts = computed(() => {
  if (activeGroup.value === 'all') return hostsStore.hosts
  return hostsStore.hosts.filter(h => h.group === activeGroup.value)
})

const availableGroups = computed(() => {
  const groups = new Set(hostsStore.hosts.map(host => host.group).filter(Boolean))
  return Array.from(groups)
})

async function handleSubmit() {
  if (!formRef.value) return

  try {
    saving.value = true
    await formRef.value.validate()
    await hostsStore.addHost(formModel.value)
    message.success('Host added successfully')
    showModal.value = false
    formModel.value = {
      name: '',
      host: '',
      port: 22,
      username: '',
      password: '',
      group: ''
    }
  } catch (error) {
    message.error('Failed to add host')
  } finally {
    saving.value = false
  }
}

async function handleConnect(host: Host) {
  connecting.value = true
  activeHost.value = host
  try {
    const terminalId = await terminalsStore.createTerminal({
      host: host.host,
      port: host.port,
      username: host.username,
      password: hostsStore.getHostPassword(host)
    })
    
    router.push({ name: 'terminal', params: { id: terminalId } })
  } catch (error) {
    message.error(`Connection failed: ${error}`)
    console.error('Connection error:', error)
  } finally {
    connecting.value = false
    activeHost.value = null
  }
}

async function handleDelete(host: Host) {
  try {
    await hostsStore.deleteHost(host.id)
    message.success('Host deleted successfully')
  } catch (error) {
    message.error('Failed to delete host')
  }
}
</script>