<template>
  <div ref="xtermRef" class="terminal-container">
    <div v-if="reconnecting" class="reconnect-overlay">
      Reconnecting... {{ reconnectAttempts }}/{{ MAX_RECONNECT_ATTEMPTS }}
    </div>
  </div>
</template>

<script lang="ts" setup>
import { ref, onMounted, onBeforeUnmount, watch } from 'vue'
import { Terminal } from '@xterm/xterm'
import { FitAddon } from '@xterm/addon-fit'
import { WebglAddon } from '@xterm/addon-webgl'
import { invoke } from '@tauri-apps/api/core'
import { useTerminalsStore } from '../../stores/terminals'
import '@xterm/xterm/css/xterm.css'

const props = defineProps<{
  sessionId: string
}>()

const emit = defineEmits<{
  error: [message: string]
  disconnect: []
  reconnect: []
}>()

// Constants
const POLLING_INTERVAL = 50
const CONNECTION_TIMEOUT = 30000
const KEEP_ALIVE_INTERVAL = 5000
const MAX_ERRORS = 5
const MAX_RECONNECT_ATTEMPTS = 3
const RECONNECT_DELAY = 2000

// Refs
const xtermRef = ref<HTMLElement | null>(null)
const terminal = ref<Terminal | null>(null)
const isPolling = ref(true)
const lastActivity = ref(Date.now())
const errorCount = ref(0)
const reconnecting = ref(false)
const reconnectAttempts = ref(0)
const terminalReady = ref(false)

// Store
const terminalsStore = useTerminalsStore()

// Addons
const fitAddon = new FitAddon()
let webglAddon: WebglAddon | null = null
let resizeObserver: ResizeObserver
const loadedAddons = ref(new Set<string>())

// Connection check interval
let connectionCheckInterval: number | null = null
let dataPollingInterval: number | null = null
let keepAliveInterval: number | null = null

// Buffer for handling partial UTF-8 sequences
let dataBuffer = ''

async function handleConnectionError(error: unknown) {
  const errorMessage = error instanceof Error ? error.message : String(error)
  console.log(error)
  if (errorMessage.includes('Connection lost') || 
      errorMessage.includes('Session not found') ||
      errorMessage.includes('Connection timeout')) {
    
    if (reconnectAttempts.value < MAX_RECONNECT_ATTEMPTS) {
      reconnecting.value = true
      reconnectAttempts.value++
      
      try {
        // Attempt to recreate the connection using stored credentials
        const terminal = terminalsStore.getTerminalBySessionId(props.sessionId)
        if (terminal) {
          await terminalsStore.reconnectTerminal(terminal.id)
          emit('reconnect')
          reconnecting.value = false
          reconnectAttempts.value = 0
          return
        }
      } catch (reconnectError) {
        console.error('Reconnection failed:', reconnectError)
      }
    }
    
    isPolling.value = false
    emit('error', 'Connection lost')
    emit('disconnect')
  }
}

async function pollData() {
  if (!isPolling.value || !terminal.value) return
  
  try {
    const data = await invoke<string>('read_ssh_data', { sessionId: props.sessionId })
    
    if (data) {
      // Combine with any buffered data
      dataBuffer += data
      
      // Attempt to process complete UTF-8 sequences
      try {
        const decodedData = new TextDecoder().decode(new TextEncoder().encode(dataBuffer))
        terminal.value.write(decodedData)
        dataBuffer = '' // Clear buffer after successful write
        lastActivity.value = Date.now()
        errorCount.value = 0
      } catch (e) {
        // Keep partial data in buffer
        console.warn('Incomplete UTF-8 sequence received')
      }
    }
  } catch (error) {
    console.error('Poll data error:', error)
    errorCount.value++
    
    if (errorCount.value >= MAX_ERRORS) {
      await handleConnectionError(error)
    }
  }
}

async function sendKeepAlive() {
  if (!isPolling.value) return
  
  try {
    await invoke('send_ssh_data', { sessionId: props.sessionId, data: '\0' })
    lastActivity.value = Date.now()
  } catch (error) {
    console.error('Keep-alive error:', error)
    await handleConnectionError(error)
  }
}

async function sendData(data: string) {
  if (!isPolling.value || Date.now() - lastActivity.value > CONNECTION_TIMEOUT) {
    await handleConnectionError(new Error('Connection timeout'))
    return
  }

  try {
    await invoke('send_ssh_data', { sessionId: props.sessionId, data })
    lastActivity.value = Date.now()
  } catch (error) {
    console.error('Send data error:', error)
    await handleConnectionError(error)
  }
}

function setupResizeHandler() {
  if (!xtermRef.value || !terminal.value) return

  resizeObserver = new ResizeObserver(async () => {
    if (!terminal.value || !terminalReady.value) return
    
    try {
      fitAddon.fit()
      const { rows, cols } = terminal.value
      await invoke('resize_pty', { 
        sessionId: props.sessionId,
        rows,
        cols
      })
    } catch (error) {
      console.error('Resize error:', error)
      await handleConnectionError(error)
    }
  })

  resizeObserver.observe(xtermRef.value)
}

function setupDataHandling() {
  if (!terminal.value) return

  terminal.value.onData((data: string) => {
    sendData(data).catch(console.error)
  })

  // Set up intervals for polling and keep-alive
  dataPollingInterval = window.setInterval(pollData, POLLING_INTERVAL)
  keepAliveInterval = window.setInterval(sendKeepAlive, KEEP_ALIVE_INTERVAL)
}

async function initializeTerminal() {
  if (!xtermRef.value) return

  try {
    terminal.value = new Terminal({
      cursorBlink: true,
      fontSize: 14,
      theme: {
        background: '#1e1e1e',
        foreground: '#d4d4d4'
      },
      scrollback: 10000,
      allowTransparency: true,
      fastScrollModifier: 'alt',
      cursorStyle: 'block'
    })

    terminal.value.open(xtermRef.value)
    
    // Load addons
    terminal.value.loadAddon(fitAddon)
    loadedAddons.value.add('fit')
    
    try {
      webglAddon = new WebglAddon()
      terminal.value.loadAddon(webglAddon)
      loadedAddons.value.add('webgl')
    } catch (e) {
      console.warn('WebGL rendering not available:', e)
    }

    await new Promise<void>((resolve) => {
      setTimeout(() => {
        if (terminal.value) {
          fitAddon.fit()
          terminalReady.value = true
          setupResizeHandler()
          setupDataHandling()
          resolve()
        }
      }, 100)
    })

  } catch (error) {
    console.error('Terminal initialization failed:', error)
    await cleanup()
    emit('error', 'Failed to initialize terminal')
    emit('disconnect')
  }
}

async function cleanup() {
  console.log('Starting cleanup...')
  isPolling.value = false
  terminalReady.value = false

  // Clear intervals
  if (connectionCheckInterval) clearInterval(connectionCheckInterval)
  if (dataPollingInterval) clearInterval(dataPollingInterval)
  if (keepAliveInterval) clearInterval(keepAliveInterval)

  // Clean up resize observer
  if (resizeObserver) {
    resizeObserver.disconnect()
  }

  // Clean up addons
  if (webglAddon) {
    try {
      webglAddon.dispose()
    } catch (e) {
      console.warn('WebGL disposal error:', e)
    }
  }

  try {
    fitAddon.dispose()
  } catch (e) {
    console.warn('Fit addon disposal error:', e)
  }

  // Dispose terminal
  if (terminal.value) {
    try {
      terminal.value.dispose()
    } catch (e) {
      console.warn('Terminal disposal error:', e)
    }
    terminal.value = null
  }

  // Close SSH connection
  try {
    await invoke('close_ssh_connection', { sessionId: props.sessionId })
  } catch (error) {
    console.error('SSH connection closure error:', error)
  }

  console.log('Cleanup completed')
}

// Watch for prop changes
watch(() => props.sessionId, () => {
  cleanup().then(() => initializeTerminal())
})

onMounted(() => {
  initializeTerminal()
  
  // Set up connection check interval
  connectionCheckInterval = window.setInterval(() => {
    if (terminalReady.value && Date.now() - lastActivity.value > CONNECTION_TIMEOUT) {
      handleConnectionError(new Error('Connection timeout'))
    }
  }, 5000)
})

onBeforeUnmount(cleanup)
</script>

<style scoped>
.terminal-container {
  height: 100%;
  width: 100%;
  background-color: #1e1e1e;
  display: flex;
  flex-direction: column;
  border-radius: 6px;
  overflow: hidden;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.15);
  position: relative;
}

.reconnect-overlay {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-color: rgba(0, 0, 0, 0.7);
  color: white;
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

:deep(.terminal) {
  flex: 1;
  min-height: 0;
}

:deep(.xterm-viewport) {
  background-color: #1e1e1e !important;
}

.xterm {
  padding: 12px;
}
</style>