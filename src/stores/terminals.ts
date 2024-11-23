import { defineStore } from 'pinia'
import { invoke } from '@tauri-apps/api/core'

interface TerminalSession {
  id: string
  name: string
  host: string
  username: string
  sessionId: string
  state: string // Add state to store terminal output
}

export const useTerminalsStore = defineStore('terminals', {
  state: () => ({
    terminals: [] as TerminalSession[],
    activeSessionId: null as string | null
  }),

  actions: {
    async createTerminal(config: { host: string; port: number; username: string; password?: string }) {
      try {
        const sessionId = await invoke<string>('create_ssh_connection', { config })
        
        const terminal: TerminalSession = {
          id: crypto.randomUUID(),
          name: `${config.username}@${config.host}`,
          host: config.host,
          username: config.username,
          sessionId,
          state: '' // Initialize state
        }
        
        this.terminals.push(terminal)
        this.activeSessionId = terminal.sessionId
        return terminal.id
      } catch (error) {
        console.error('Failed to create terminal:', error)
        throw error
      }
    },

    async closeTerminal(id: string) {
      const terminal = this.getTerminal(id)
      if (!terminal) return

      try {
        await invoke('close_ssh_connection', { sessionId: terminal.sessionId })
      } catch (error) {
        console.error('Error closing terminal:', error)
      } finally {
        this.terminals = this.terminals.filter(t => t.id !== id)
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
      }
    }
  }
})