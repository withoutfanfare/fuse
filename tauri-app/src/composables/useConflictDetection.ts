import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { ConflictStatus } from '../types'

/**
 * Composable for detecting merge conflicts on a pull request.
 *
 * Calls the `check_merge_conflicts` Tauri command and exposes
 * the resulting conflict status, loading state, and any error.
 */
export function useConflictDetection() {
  const conflictStatus = ref<ConflictStatus | null>(null)
  const loading = ref(false)
  const error = ref<string | null>(null)

  async function checkConflicts(prId: number) {
    loading.value = true
    error.value = null
    try {
      conflictStatus.value = await invoke<ConflictStatus>('check_merge_conflicts', { prId })
    } catch (e) {
      error.value = String(e)
      conflictStatus.value = null
    } finally {
      loading.value = false
    }
  }

  return { conflictStatus, loading, error, checkConflicts }
}
