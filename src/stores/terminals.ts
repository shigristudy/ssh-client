import { defineStore } from 'pinia'
import { invoke } from '@tauri-apps/api/core'

interface TerminalSession {
  id: string
  name: string
  host: string
  port: number
  username: string
  sessionId: string
  state: string
  lastConnected: Date
  reconnectAttempts: number
  isConnected: boolean
  lastActivity: Date
  bufferState: string
  password?: string
  privateKey?: string
}

interface TerminalConfig {
  host: string
  port: number
  username: string
  password?: string
  privateKey?: string
}

export const useTerminalsStore = defineStore('terminals', {
  state: () => ({
    terminals: [] as TerminalSession[],
    activeSessionId: null as string | null,
    connectionConfigs: new Map<string, TerminalConfig>(),
    errors: new Map<string, string[]>()
  }),

  getters: {
    activeTerminal: (state) => {
      return state.terminals.find(t => t.sessionId === state.activeSessionId)
    },
    
    connectedTerminals: (state) => {
      return state.terminals.filter(t => t.isConnected)
    },
    
    terminalErrors: (state) => (id: string) => {
      return state.errors.get(id) || []
    }
  },

  actions: {
    async createTerminal(config: TerminalConfig) {
      try {
        const sessionId = await invoke<string>('create_ssh_connection', { config })
        
        const terminal: TerminalSession = {
          id: crypto.randomUUID(),
          name: `${config.username}@${config.host}`,
          host: config.host,
          port: config.port,
          username: config.username,
          sessionId,
          state: '',
          lastConnected: new Date(),
          reconnectAttempts: 0,
          isConnected: true,
          lastActivity: new Date(),
          bufferState: '',
          password: config.password,
          privateKey: config.privateKey
        }
        
        // Store config for potential reconnection (remove sensitive data)
        const savedConfig = { ...config }
        delete savedConfig.password
        delete savedConfig.privateKey
        this.connectionConfigs.set(terminal.id, savedConfig)
        
        this.terminals.push(terminal)
        this.activeSessionId = terminal.sessionId
        return terminal.id
      } catch (error) {
        this.addError(error)
        throw error
      }
    },

    async reconnectTerminal(id: string) {
      const terminal = this.getTerminal(id)
      const config = this.connectionConfigs.get(id)
      
      if (!terminal || !config) {
        throw new Error('Terminal configuration not found')
      }

      // Don't exceed max reconnection attempts
      if (terminal.reconnectAttempts >= 3) {
        throw new Error('Maximum reconnection attempts exceeded')
      }

      terminal.reconnectAttempts++
      
      const fullConfig = {
        ...config,
        password: terminal.password,
        privateKey: terminal.privateKey
      }
      
      try {
        const sessionId = await invoke<string>('create_ssh_connection', { config: fullConfig })
        terminal.sessionId = sessionId
        terminal.lastConnected = new Date()
        terminal.reconnectAttempts = 0
        terminal.isConnected = true
        terminal.lastActivity = new Date()
        this.clearErrors(id)
        return sessionId
      } catch (error) {
        this.addError(error, id)
        throw error
      }
    },

    async closeTerminal(id: string) {
      const terminal = this.getTerminal(id)
      if (!terminal) return

      try {
        await invoke('close_ssh_connection', { sessionId: terminal.sessionId })
        terminal.isConnected = false
      } catch (error) {
        console.error('Error closing terminal:', error)
      } finally {
        this.terminals = this.terminals.filter(t => t.id !== id)
        this.connectionConfigs.delete(id)
        this.errors.delete(id)
        if (this.activeSessionId === terminal.sessionId) {
          this.activeSessionId = null
        }
      }
    },

    getTerminal(id: string) {
      return this.terminals.find(t => t.id === id)
    },

    getTerminalBySessionId(sessionId: string) {
      return this.terminals.find(t => t.sessionId === sessionId)
    },

    updateTerminalState(sessionId: string, data: string) {
      const terminal = this.getTerminalBySessionId(sessionId)
      if (terminal) {
        terminal.state += data
        terminal.lastActivity = new Date()
      }
    },

    updateBufferState(sessionId: string, data: string) {
      const terminal = this.getTerminalBySessionId(sessionId)
      if (terminal) {
        terminal.bufferState = data
      }
    },

    setTerminalDisconnected(sessionId: string) {
      const terminal = this.getTerminalBySessionId(sessionId)
      if (terminal) {
        terminal.isConnected = false
      }
    },

    addError(error: unknown, id?: string) {
      const errorMessage = error instanceof Error ? error.message : String(error)
      if (id) {
        const terminalErrors = this.errors.get(id) || []
        terminalErrors.push(errorMessage)
        this.errors.set(id, terminalErrors)
      }
    },

    clearErrors(id: string) {
      this.errors.delete(id)
    },

    // Cleanup disconnected sessions
    cleanupDisconnectedSessions() {
      const now = new Date()
      this.terminals = this.terminals.filter(terminal => {
        const timeSinceLastActivity = now.getTime() - terminal.lastActivity.getTime()
        return terminal.isConnected || timeSinceLastActivity < 300000 // 5 minutes
      })
    }
  }
})