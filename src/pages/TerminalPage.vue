<template>
  <div class="terminal-page">
    <Terminal 
      v-if="terminal" 
      :session-id="terminal.sessionId"
      @error="handleError"
      @disconnect="handleDisconnect"
    />
    <n-result 
      v-else 
      status="404" 
      title="Terminal not found" 
      description="The requested terminal session does not exist"
    >
      <template #footer>
        <n-button @click="router.push({ name: 'hosts' })">
          Back to Hosts
        </n-button>
      </template>
    </n-result>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { NResult, NButton, useMessage } from 'naive-ui'
import Terminal from '../components/ssh/Terminal.vue'
import { useTerminalsStore } from '../stores/terminals'

const route = useRoute()
const router = useRouter()
const message = useMessage()
const terminalsStore = useTerminalsStore()
const disconnecting = ref(false)

const terminal = computed(() => {
  return terminalsStore.getTerminal(route.params.id as string)
})

function handleError(errorMessage: string) {
  message.error(`Terminal error: ${errorMessage}`)
}

async function handleDisconnect() {
  if (disconnecting.value || !terminal.value) return
  disconnecting.value = true
  
  try {
    await terminalsStore.closeTerminal(terminal.value.id)
    message.warning('Terminal session closed')
    router.push({ name: 'hosts' })
  } catch (error) {
    console.error('Error closing terminal:', error)
  } finally {
    disconnecting.value = false
  }
}
</script>

<style scoped>
.terminal-page {
  height: 100vh;
  width: 100%;
  display: flex;
  flex-direction: column;
  background: #1e1e1e;
  padding: 0;
  box-sizing: border-box;
  overflow: hidden;
}
</style>