import { defineStore } from 'pinia'

export interface LogEntry {
  timestamp: number
  level: 'info' | 'error' | 'warn' | 'debug'
  source: 'frontend' | 'backend'
  message: string
}

export const useLogsStore = defineStore('logs', {
  state: () => ({
    logs: [] as LogEntry[],
    maxLogs: 1000, // Maximum number of logs to keep
    isVisible: false
  }),

  actions: {
    addLog(entry: Omit<LogEntry, 'timestamp'>) {
      this.logs.push({
        ...entry,
        timestamp: Date.now()
      })

      // Trim logs if they exceed maxLogs
      if (this.logs.length > this.maxLogs) {
        this.logs = this.logs.slice(-this.maxLogs)
      }
    },

    clearLogs() {
      this.logs = []
    },

    toggleVisibility() {
      this.isVisible = !this.isVisible
    },

    addBackendLog(message: string, level: 'info' | 'error' | 'warn' | 'debug' = 'info') {
      this.addLog({
        level,
        source: 'backend',
        message
      })
    }
  }
})
