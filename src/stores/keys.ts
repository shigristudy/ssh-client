import { defineStore } from 'pinia'

interface SshKey {
  id: string;
  name: string;
  privateKey: string;
  publicKey?: string;
}

export const useKeysStore = defineStore('keys', {
  state: () => ({
    keys: [] as SshKey[]
  }),

  actions: {
    async addKey(key: Omit<SshKey, 'id'>) {
      const newKey: SshKey = {
        ...key,
        id: crypto.randomUUID()
      }
      this.keys.push(newKey)
      await this.saveKeys()
    },

    async saveKeys() {
      localStorage.setItem('ssh_keys', JSON.stringify(this.keys))
    },

    async loadKeys() {
      const stored = localStorage.getItem('ssh_keys')
      if (stored) {
        this.keys = JSON.parse(stored)
      }
    }
  }
})
