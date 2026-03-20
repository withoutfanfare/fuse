import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { PriorityQueueItem } from '../types'

/**
 * Composable for the smart review queue with priority scoring.
 * Fetches open PRs sorted by computed priority (highest first).
 * The first item is the "next to review" suggestion.
 */
export function usePriorityQueue() {
  const queue = ref<PriorityQueueItem[]>([])
  const loading = ref(false)
  const error = ref<string | null>(null)

  async function fetchQueue() {
    loading.value = true
    error.value = null
    try {
      queue.value = await invoke<PriorityQueueItem[]>('get_priority_queue')
    } catch (e) {
      error.value = String(e)
    } finally {
      loading.value = false
    }
  }

  return { queue, loading, error, fetchQueue }
}
