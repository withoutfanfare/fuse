import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { ReviewerWorkloadStats } from '../types'

/**
 * Composable for fetching and managing reviewer workload statistics.
 * Aggregates assigned, completed, and overdue review counts per reviewer.
 */
export function useWorkload() {
  const workload = ref<ReviewerWorkloadStats[]>([])
  const loading = ref(false)
  const error = ref<string | null>(null)

  async function fetchWorkload() {
    loading.value = true
    error.value = null
    try {
      workload.value = await invoke<ReviewerWorkloadStats[]>('get_reviewer_workload')
    } catch (e) {
      error.value = String(e)
    } finally {
      loading.value = false
    }
  }

  return { workload, loading, error, fetchWorkload }
}
