import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { ReviewDigest } from '../types'

/**
 * Composable for fetching the recurring review digest.
 * Aggregates PR stats for a given period and compares against the previous period.
 */
export function useReviewDigest() {
  const digest = ref<ReviewDigest | null>(null)
  const loading = ref(false)
  const error = ref<string | null>(null)

  async function fetchDigest(days: number = 7) {
    loading.value = true
    error.value = null
    try {
      digest.value = await invoke<ReviewDigest>('get_review_digest', { days })
    } catch (e) {
      error.value = String(e)
    } finally {
      loading.value = false
    }
  }

  return { digest, loading, error, fetchDigest }
}
