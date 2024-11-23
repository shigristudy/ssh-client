<template>
  <div class="settings-page">
    <n-space vertical :size="24">
      <n-card title="General Settings">
        <n-form :model="generalSettings" label-placement="left" label-width="160">
          <n-form-item label="Default Port">
            <n-input-number v-model:value="generalSettings.defaultPort" :min="1" :max="65535" />
          </n-form-item>
          <n-form-item label="Default Username">
            <n-input v-model:value="generalSettings.defaultUsername" />
          </n-form-item>
          <n-form-item label="Theme">
            <n-select v-model:value="generalSettings.theme" :options="themeOptions" />
          </n-form-item>
        </n-form>
      </n-card>

      <n-card title="GitHub Settings">
        <n-space vertical>
          <n-alert v-if="!form.githubToken" type="warning">
            GitHub token is required for syncing hosts. You can create one at 
            <a href="https://github.com/settings/tokens" target="_blank">GitHub Settings</a>.
          </n-alert>
          
          <n-form :model="form" label-placement="left" label-width="160">
            <n-form-item label="GitHub Token" required>
              <n-input-group>
                <n-input
                  v-model:value="form.githubToken"
                  type="password"
                  placeholder="Enter your GitHub token"
                  :disabled="syncing"
                />
                <n-button
                  type="primary"
                  :loading="testing"
                  :disabled="!form.githubToken"
                  @click="testConnection"
                >
                  Test
                </n-button>
              </n-input-group>
            </n-form-item>

            <n-form-item label="Sync Status">
              <n-space align="center">
                <n-tag :type="lastSync ? 'success' : 'warning'">
                  {{ lastSync ? `Last synced: ${new Date(lastSync).toLocaleString()}` : 'Never synced' }}
                </n-tag>
                <n-button
                  type="primary"
                  :loading="syncing"
                  :disabled="!form.githubToken"
                  @click="syncWithGithub"
                >
                  Sync Now
                </n-button>
              </n-space>
            </n-form-item>
          </n-form>

          <template #footer>
            <p class="help-text">
              To get a GitHub token, go to 
              <a href="https://github.com/settings/tokens" target="_blank">
                GitHub Settings > Developer settings > Personal access tokens
              </a>
              and create a token with "gist" scope.
            </p>
          </template>
        </n-space>
      </n-card>
    </n-space>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, watch } from 'vue'
import { 
  NCard, NForm, NFormItem, NInput, NInputNumber, NSelect, 
  NButton, NSpace, NTag, NAlert, NInputGroup, useMessage 
} from 'naive-ui'
import { useSettingsStore } from '../stores/settings'
import { useHostsStore } from '../stores/hosts'

const message = useMessage()
const settingsStore = useSettingsStore()
const hostsStore = useHostsStore()

const testing = ref(false)
const syncing = ref(false)
const lastSync = ref<number | null>(settingsStore.lastSync)

const generalSettings = reactive({
  defaultPort: settingsStore.defaultPort,
  defaultUsername: settingsStore.defaultUsername,
  theme: settingsStore.theme
})

const form = reactive({
  githubToken: settingsStore.githubToken
})

const themeOptions = [
  { label: 'Light', value: 'light' },
  { label: 'Dark', value: 'dark' }
]

// Auto-save general settings when changed
watch(generalSettings, async (newSettings) => {
  Object.assign(settingsStore.$state, newSettings)
  await settingsStore.saveSettings()
}, { deep: true })

async function testConnection() {
  testing.value = true
  try {
    const success = await settingsStore.testGithubConnection(form.githubToken)
    if (success) {
      message.success('GitHub connection successful')
      settingsStore.githubToken = form.githubToken
      await settingsStore.saveSettings()
    } else {
      message.error('GitHub connection failed')
    }
  } catch (error) {
    message.error('GitHub connection failed')
  } finally {
    testing.value = false
  }
}

async function syncWithGithub() {
  syncing.value = true
  try {
    await settingsStore.syncHosts()
    message.success('Successfully synced with GitHub')
    lastSync.value = Date.now()
  } catch (error) {
    console.error('Sync error:', error)
    message.error('Failed to sync with GitHub')
  } finally {
    syncing.value = false
  }
}
</script>

<style scoped>
.settings-page {
  padding: 24px;
  max-width: 800px;
  margin: 0 auto;
  height: 100%;
  overflow-y: auto;
}

.help-text {
  font-size: 0.9em;
  color: #666;
  margin: 0;
}

.help-text a {
  color: #18a058;
  text-decoration: none;
}

.help-text a:hover {
  text-decoration: underline;
}
</style>