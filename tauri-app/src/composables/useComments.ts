import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { PrComment, PrReviewComment, PrCommentsResponse } from '../types'

export function useComments() {
  const comments = ref<PrComment[]>([])
  const reviews = ref<PrReviewComment[]>([])
  const loading = ref(false)
  const error = ref<string | null>(null)

  async function fetchComments(prId: number) {
    loading.value = true
    error.value = null
    try {
      const response = await invoke<PrCommentsResponse>('fetch_pr_comments', { prId })
      comments.value = response.comments
      reviews.value = response.reviews
    } catch (e) {
      error.value = String(e)
    } finally {
      loading.value = false
    }
  }

  return { comments, reviews, loading, error, fetchComments }
}
