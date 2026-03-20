import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export const useSettingsStore = defineStore('settings', () => {
  const settings = ref<Record<string, string>>({})
  const loading = ref(false)
  const error = ref<string | null>(null)

  async function fetchSettings() {
    loading.value = true
    error.value = null
    try {
      settings.value = await invoke<Record<string, string>>('get_settings')
    } catch (e) {
      error.value = String(e)
    } finally {
      loading.value = false
    }
  }

  async function updateSetting(key: string, value: string) {
    error.value = null
    try {
      await invoke('update_setting', { key, value })
      settings.value[key] = value
    } catch (e) {
      error.value = String(e)
      throw e
    }
  }

  return { settings, loading, error, fetchSettings, updateSetting }
})
