
import { Octokit } from '@octokit/rest'
import { encrypt, decrypt } from '../utils/crypto'

export class GitHubService {
  private octokit: Octokit
  private gistId: string | null = null

  constructor(token: string) {
    this.octokit = new Octokit({ auth: token })
  }

  async saveConfig(data: any) {
    const encrypted = encrypt(JSON.stringify(data))
    
    if (this.gistId) {
      await this.updateGist(encrypted)
    } else {
      await this.createGist(encrypted)
    }
  }

  async loadConfig() {
    const gists = await this.octokit.gists.list()
    const configGist = gists.data.find(g => g.description === 'SSH-Client-Config')
    
    if (configGist) {
      this.gistId = configGist.id
      const content = Object.values(configGist.files)[0].content
      if (content) {
        return JSON.parse(decrypt(content))
      }
    }
    return null
  }

  private async createGist(content: string) {
    const response = await this.octokit.gists.create({
      description: 'SSH-Client-Config',
      public: false,
      files: {
        'config.enc': {
          content
        }
      }
    })
    this.gistId = response.data.id
  }

  private async updateGist(content: string) {
    if (!this.gistId) throw new Error('No gist ID')
    
    await this.octokit.gists.update({
      gist_id: this.gistId,
      files: {
        'config.enc': {
          content
        }
      }
    })
  }
}