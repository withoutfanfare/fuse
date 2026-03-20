<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { useAiReview } from '../composables/useAiReview'
import type { PullRequest } from '../types'
import MarkdownRenderer from './MarkdownRenderer.vue'
import AiReviewComparison from './AiReviewComparison.vue'

const props = defineProps<{
  pr: PullRequest
}>()

const { reviews, reviewing, error, fetchReviews, triggerReview } = useAiReview()

const hasReviews = computed(() => reviews.value.length > 0)
const canCompare = computed(() => reviews.value.length >= 2)
const comparisonExpanded = ref(false)

async function handleRunReview() {
  await triggerReview(props.pr.id)
}

function formatDate(dateStr: string): string {
  return new Date(dateStr).toLocaleDateString('en-GB', {
    day: 'numeric', month: 'short', year: 'numeric',
    hour: '2-digit', minute: '2-digit',
  })
}

onMounted(() => {
  fetchReviews(props.pr.id)
})
</script>

<template>
  <div class="ai-review-panel">
    <div class="panel-header">
      <h2 class="section-title">AI Review</h2>
      <button
        class="btn-run-review"
        :disabled="reviewing"
        @click="handleRunReview"
      >
        {{ reviewing ? 'Reviewing…' : 'Run Review' }}
      </button>
    </div>

    <div v-if="reviewing" class="review-loading">
      <div class="loading-indicator">
        <span class="loading-dot" />
        <span class="loading-dot" />
        <span class="loading-dot" />
      </div>
      <p class="loading-text">Creating worktree and running Claude… this may take up to 60 seconds</p>
    </div>

    <div v-if="error" class="review-error">
      {{ error }}
    </div>

    <div v-if="hasReviews" class="reviews-list">
      <!-- Review comparison toggle — shown when 2+ reviews exist -->
      <div v-if="canCompare" class="comparison-toggle-wrapper">
        <button
          class="btn-compare"
          @click="comparisonExpanded = !comparisonExpanded"
        >
          {{ comparisonExpanded ? 'Hide Comparison' : 'Compare Reviews' }}
          <span class="compare-count">{{ reviews.length }} reviews</span>
        </button>
      </div>

      <!-- AI Review Comparison panel -->
      <div v-if="canCompare && comparisonExpanded" class="comparison-container">
        <AiReviewComparison :reviews="reviews" />
      </div>

      <details
        v-for="review in reviews"
        :key="review.id"
        class="review-entry"
        :open="reviews.indexOf(review) === 0"
      >
        <summary class="review-summary">
          <span class="review-date">{{ formatDate(review.created_at) }}</span>
          <span class="review-branch">{{ review.worktree_branch }}</span>
        </summary>
        <div class="review-content">
          <MarkdownRenderer :content="review.review_text" />
        </div>
      </details>
    </div>

    <div v-else-if="!reviewing && !error" class="review-empty">
      No AI reviews yet. Click "Run Review" to generate one.
    </div>
  </div>
</template>

<style scoped>
.ai-review-panel {
  display: flex;
  flex-direction: column;
  gap: var(--space-4);
}

.panel-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.panel-header .section-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--color-text-secondary);
  margin: 0;
}

.btn-run-review {
  background: rgba(59, 130, 246, 0.2);
  color: var(--color-status-info);
  font-weight: 600;
  font-size: 13px;
  padding: var(--space-2) var(--space-4);
  border-radius: var(--radius-md);
  border: 1px solid rgba(59, 130, 246, 0.3);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.btn-run-review:hover:not(:disabled) {
  background: rgba(59, 130, 246, 0.3);
  border-color: rgba(59, 130, 246, 0.5);
}

.btn-run-review:active:not(:disabled) {
  transform: scale(0.97);
}

.btn-run-review:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.review-loading {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--space-3);
  padding: var(--space-6);
}

.loading-indicator {
  display: flex;
  gap: var(--space-2);
}

.loading-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: var(--color-status-info);
  animation: pulse 1.4s ease-in-out infinite;
}

.loading-dot:nth-child(2) { animation-delay: 0.2s; }
.loading-dot:nth-child(3) { animation-delay: 0.4s; }

@keyframes pulse {
  0%, 80%, 100% { opacity: 0.3; transform: scale(0.8); }
  40% { opacity: 1; transform: scale(1); }
}

.loading-text {
  font-size: 13px;
  color: var(--color-text-muted);
  text-align: center;
}

.review-error {
  background: rgba(220, 38, 38, 0.1);
  border: 1px solid rgba(220, 38, 38, 0.3);
  border-radius: var(--radius-md);
  padding: var(--space-3);
  font-size: 13px;
  color: var(--color-status-danger);
}

.reviews-list {
  display: flex;
  flex-direction: column;
  gap: var(--space-3);
}

.review-entry {
  border: 1px solid var(--color-border-default);
  border-radius: var(--radius-md);
  overflow: hidden;
}

.review-summary {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  padding: var(--space-3) var(--space-4);
  cursor: pointer;
  font-size: 13px;
  transition: background var(--transition-fast);
  user-select: none;
}

.review-summary:hover {
  background: rgba(255, 255, 255, 0.02);
}

.review-date {
  color: var(--color-text-secondary);
  font-weight: 500;
}

.review-branch {
  background: rgba(59, 130, 246, 0.15);
  color: var(--color-status-info);
  font-size: 11px;
  font-family: var(--font-mono);
  padding: 1px var(--space-2);
  border-radius: var(--radius-sm);
}

.review-content {
  padding: var(--space-4);
  border-top: 1px solid var(--color-border-default);
}

.review-empty {
  text-align: center;
  color: var(--color-text-muted);
  font-size: 13px;
  padding: var(--space-4);
}

/* Review comparison toggle */
.comparison-toggle-wrapper {
  display: flex;
  justify-content: flex-end;
}

.btn-compare {
  display: inline-flex;
  align-items: center;
  gap: var(--space-2);
  background: rgba(139, 92, 246, 0.15);
  color: #a78bfa;
  font-weight: 600;
  font-size: 12px;
  padding: var(--space-1) var(--space-3);
  border-radius: var(--radius-md);
  border: 1px solid rgba(139, 92, 246, 0.25);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.btn-compare:hover {
  background: rgba(139, 92, 246, 0.25);
  border-color: rgba(139, 92, 246, 0.4);
}

.btn-compare:active {
  transform: scale(0.97);
}

.btn-compare:focus-visible {
  box-shadow: 0 0 0 2px var(--color-accent-muted);
  outline: none;
}

.compare-count {
  font-weight: 400;
  opacity: 0.7;
  font-size: 11px;
}

.comparison-container {
  border: 1px solid rgba(139, 92, 246, 0.2);
  border-radius: var(--radius-md);
  padding: var(--space-4);
  background: rgba(139, 92, 246, 0.04);
}
</style>
