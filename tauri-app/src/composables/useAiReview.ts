import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { AiReview } from '../types'

export function useAiReview() {
  const reviews = ref<AiReview[]>([])
  const reviewing = ref(false)
  const error = ref<string | null>(null)

  async function fetchReviews(prId: number): Promise<void> {
    try {
      reviews.value = await invoke<AiReview[]>('list_pr_ai_reviews', { prId })
    } catch (e) {
      error.value = String(e)
    }
  }

  async function triggerReview(prId: number): Promise<AiReview | null> {
    reviewing.value = true
    error.value = null
    try {
      const review = await invoke<AiReview>('trigger_worktree_review', { prId })
      reviews.value.unshift(review)
      return review
    } catch (e) {
      error.value = String(e)
      return null
    } finally {
      reviewing.value = false
    }
  }

  return { reviews, reviewing, error, fetchReviews, triggerReview }
}
