<template>
  <n-space vertical>
    <n-card>
      <n-space justify="space-between">
        <h2>SSH Keys</h2>
        <n-button type="primary" @click="showModal = true">Add Key</n-button>
      </n-space>

      <n-data-table
        :columns="columns"
        :data="keysStore.keys"
        :pagination="{ pageSize: 10 }"
      />
    </n-card>

    <n-modal v-model:show="showModal">
      <n-card style="width: 600px" title="Add SSH Key">
        <n-form ref="formRef" :model="formModel" :rules="rules">
          <n-form-item label="Name" path="name">
            <n-input v-model:value="formModel.name" placeholder="My Key" />
          </n-form-item>
          <n-form-item label="Private Key" path="privateKey">
            <n-input
              v-model:value="formModel.privateKey"
              type="textarea"
              :rows="8"
              placeholder="-----BEGIN OPENSSH PRIVATE KEY-----"
            />
          </n-form-item>
          <n-form-item label="Public Key" path="publicKey">
            <n-input
              v-model:value="formModel.publicKey"
              type="textarea"
              :rows="3"
              placeholder="ssh-rsa AAAA..."
            />
          </n-form-item>
        </n-form>
        <template #footer>
          <n-space justify="end">
            <n-button @click="showModal = false">Cancel</n-button>
            <n-button type="primary" @click="handleSubmit">Save</n-button>
          </n-space>
        </template>
      </n-card>
    </n-modal>
  </n-space>
</template>

<script setup lang="ts">
import { h, ref } from 'vue'
import { useKeysStore } from '../stores/keys'
import { useMessage } from 'naive-ui'
import type { DataTableColumns } from 'naive-ui'

const keysStore = useKeysStore()
const message = useMessage()
const showModal = ref(false)
const formRef = ref()

const formModel = ref({
  name: '',
  privateKey: '',
  publicKey: ''
})

const rules = {
  name: { required: true, message: 'Please enter a name' },
  privateKey: { required: true, message: 'Private key is required' }
}

const columns: DataTableColumns = [
  { title: 'Name', key: 'name' },
  {
    title: 'Actions',
    key: 'actions',
    render(row) {
      return h(
        'div',
        {
          style: 'display: flex; gap: 8px;'
        },
        [
          h(
            'n-button',
            {
              size: 'small',
              type: 'error',
              onClick: () => handleDeleteKey(row.id)
            },
            { default: () => 'Delete' }
          )
        ]
      )
    }
  }
]

async function handleSubmit() {
  try {
    await formRef.value?.validate()
    await keysStore.addKey(formModel.value)
    message.success('Key added successfully')
    showModal.value = false
    formModel.value = {
      name: '',
      privateKey: '',
      publicKey: ''
    }
  } catch (error) {
    message.error('Failed to add key')
  }
}

async function handleDeleteKey(id:any) {
  try {
    const index = keysStore.keys.findIndex(key => key.id === id)
    if (index > -1) {
      keysStore.keys.splice(index, 1)
      await keysStore.saveKeys()
      message.success('Key deleted successfully')
    }
  } catch (error) {
    message.error('Failed to delete key')
  }
}
</script>