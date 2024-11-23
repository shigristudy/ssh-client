<template>
  <div ref="xtermRef" class="terminal-container"></div>
</template>

<script lang="ts" setup>
import { ref, onMounted, onUnmounted, onBeforeUnmount } from 'vue'
import { Terminal } from '@xterm/xterm'
import { FitAddon } from '@xterm/addon-fit'
import { invoke } from '@tauri-apps/api/core'
import { useTerminalsStore } from '../../stores/terminals'
import '@xterm/xterm/css/xterm.css'

const props = defineProps<{
  sessionId: string
}>()

const emit = defineEmits<{
  error: [message: string]
  disconnect: []
}>()

const terminalsStore = useTerminalsStore()

const errorCount = ref(0)
const MAX_ERRORS = 3
const RETRY_DELAY = 1000 // 1 second between retries

const xtermRef = ref<HTMLElement | null>(null)
const terminal = ref<Terminal | null>(null)
const fitAddon = new FitAddon()
const isPolling = ref(true)

const POLLING_INTERVAL = 50 // Reduce polling interval to improve responsiveness
const CONNECTION_TIMEOUT = 20000 // 20 seconds
const KEEP_ALIVE_INTERVAL = 5000 // 5 seconds
const lastActivity = ref(Date.now())

async function pollData() {
  while (isPolling.value && terminal.value) {
    try {
      const data = await invoke<string>('read_ssh_data', { 
        sessionId: props.sessionId 
      })

      if (data && terminal.value) {
        terminal.value.write(data)
        terminalsStore.updateTerminalState(props.sessionId, data) // Save terminal state
        errorCount.value = 0
        lastActivity.value = Date.now()
      } else if (Date.now() - lastActivity.value > CONNECTION_TIMEOUT) {
        throw new Error('Connection timeout')
      }

      await new Promise(resolve => setTimeout(resolve, POLLING_INTERVAL))
    } catch (error) {
      console.error('Error reading SSH data:', error)
      const errorMessage = error instanceof Error ? error.message : String(error)
      
      if (errorMessage.includes('Connection lost') || 
          errorMessage.includes('Connection timeout') ||
          errorMessage.includes('Session not found')) {
        isPolling.value = false
        emit('error', 'Connection lost')
        emit('disconnect')
        break
      }

      errorCount.value++
      if (errorCount.value >= MAX_ERRORS) {
        isPolling.value = false
        emit('error', 'Connection lost due to too many errors')
        emit('disconnect')
        break
      }
      
      await new Promise(resolve => setTimeout(resolve, RETRY_DELAY))
    }
  }
}

async function sendKeepAlive() {
  while (isPolling.value) {
    try {
      await invoke('send_ssh_data', { sessionId: props.sessionId, data: '\0' })
      lastActivity.value = Date.now()
    } catch (error) {
      console.error('Failed to send keep-alive:', error)
    }
    await new Promise(resolve => setTimeout(resolve, KEEP_ALIVE_INTERVAL))
  }
}

// Add resize observer
const resizeObserver = new ResizeObserver(() => {
  if (terminal.value) {
    fitAddon.fit();
    const { rows, cols } = terminal.value
    invoke('resize_pty', { 
      sessionId: props.sessionId,
      rows,
      cols
    }).catch(console.error)
  }
});

async function sendData(data: string) {
  if (!isPolling.value || Date.now() - lastActivity.value > CONNECTION_TIMEOUT) {
    emit('error', 'Connection lost')
    emit('disconnect')
    return
  }

  try {
    await invoke('send_ssh_data', { sessionId: props.sessionId, data })
    lastActivity.value = Date.now()
  } catch (error) {
    console.error('Failed to send data:', error)
    const errorMessage = error instanceof Error ? error.message : String(error)
    emit('error', errorMessage)
    if (errorMessage.includes('Connection lost') || 
        errorMessage.includes('Session not found')) {
      emit('disconnect')
    }
  }
}

async function initializeTerminal() {
  if (!xtermRef.value) return;
  
  terminal.value = new Terminal({
    cursorBlink: true,
    fontFamily: 'MesloLGS NF, Menlo, Monaco, "Courier New", monospace',
    fontSize: 14,
    lineHeight: 1.2,
    theme: {
      background: '#1e1e1e',
      foreground: '#d4d4d4',
      cursor: '#d4d4d4',
      black: '#000000',
      red: '#cd3131',
      green: '#0dbc79',
      yellow: '#e5e510',
      blue: '#2472c8',
      magenta: '#bc3fbc',
      cyan: '#11a8cd',
      white: '#e5e5e5',
      brightBlack: '#666666',
      brightRed: '#f14c4c',
      brightGreen: '#23d18b',
      brightYellow: '#f5f543',
      brightBlue: '#3b8eea',
      brightMagenta: '#d670d6',
      brightCyan: '#29b8db',
      brightWhite: '#e5e5e5'
    },
    scrollback: 10000, // Increase scrollback buffer
    windowsMode: false,
    convertEol: true,
    disableStdin: false,
    allowProposedApi: true,
    allowTransparency: true,
    fastScrollModifier: 'alt',
    fastScrollSensitivity: 5,
    scrollSensitivity: 1,
    macOptionIsMeta: true,
    macOptionClickForcesSelection: true,
    rightClickSelectsWord: true,
  });

  terminal.value.loadAddon(fitAddon);
  terminal.value.open(xtermRef.value);
  fitAddon.fit();

  // Restore terminal state
  const terminalSession = terminalsStore.getTerminalBySessionId(props.sessionId)
  if (terminalSession && terminalSession.state) {
    terminal.value.write(terminalSession.state)
  }

  // Send initial resize
  const { rows, cols } = terminal.value;
  await invoke('resize_pty', { 
    sessionId: props.sessionId,
    rows,
    cols
  });

  terminal.value.onData(async (data) => {
    await sendData(data)
  });

  resizeObserver.observe(xtermRef.value);
  
  pollData();
  sendKeepAlive();
}

async function cleanup() {
  isPolling.value = false
  resizeObserver.disconnect()
  
  if (terminal.value) {
    terminal.value.dispose()
    terminal.value = null
  }
  
  try {
    await invoke('close_ssh_connection', { 
      sessionId: props.sessionId 
    })
  } catch (error) {
    console.error('Error closing SSH connection:', error)
  }
}

onMounted(() => {
  initializeTerminal()
})

onUnmounted(() => {
  cleanup()
})

// Define an explicit onBeforeUnmount handler for cleanup
onBeforeUnmount(() => {
  cleanup()
})
</script>

<style>
/* Remove global terminal styles that affect scrolling */
.xterm {
  padding: 12px;
}
</style>

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
}

:deep(.terminal) {
  flex: 1;
  min-height: 0; /* Important for flexbox scrolling */
}

:deep(.xterm-viewport) {
  background-color: #1e1e1e !important;
}
</style>