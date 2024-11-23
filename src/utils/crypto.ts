
import CryptoJS from 'crypto-js'
import { invoke } from '@tauri-apps/api/core'

const MASTER_KEY = await invoke<string>('get_master_key')

export function encrypt(data: string): string {
  return CryptoJS.AES.encrypt(data, MASTER_KEY).toString()
}

export function decrypt(encrypted: string): string {
  const bytes = CryptoJS.AES.decrypt(encrypted, MASTER_KEY)
  return bytes.toString(CryptoJS.enc.Utf8)
}

export async function generateKeyPair(): Promise<{publicKey: string, privateKey: string}> {
  return await invoke('generate_ssh_keypair')
}

export async function getKeyFingerprint(publicKey: string): Promise<string> {
  return await invoke('get_key_fingerprint', { publicKey })
}