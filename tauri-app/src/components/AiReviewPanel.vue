<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { SButton, SSpinner, SBadge, SCard, SEmptyState, SNoticeBanner } from '@stuntrocket/ui'
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
      <SButton
        variant="primary"
        size="sm"
        :disabled="reviewing"
        :loading="reviewing"
        @click="handleRunReview"
      >
        {{ reviewing ? 'Reviewing...' : 'Run Review' }}
      </SButton>
    </div>

    <div v-if="reviewing" class="review-loading">
      <SSpinner />
      <p class="loading-text">Creating worktree and running Claude... this may take up to 60 seconds</p>
    </div>

    <SNoticeBanner v-if="error" variant="danger">
      {{ error }}
    </SNoticeBanner>

    <div v-if="hasReviews" class="reviews-list">
      <!-- Review comparison toggle — shown when 2+ reviews exist -->
      <div v-if="canCompare" class="comparison-toggle-wrapper">
        <SButton
          variant="ghost"
          size="sm"
          @click="comparisonExpanded = !comparisonExpanded"
        >
          {{ comparisonExpanded ? 'Hide Comparison' : 'Compare Reviews' }}
          <SBadge variant="count">{{ reviews.length }} reviews</SBadge>
        </SButton>
      </div>

      <!-- AI Review Comparison panel -->
      <SCard v-if="canCompare && comparisonExpanded" variant="nested">
        <AiReviewComparison :reviews="reviews" />
      </SCard>

      <details
        v-for="review in reviews"
        :key="review.id"
        class="review-entry"
        :open="reviews.indexOf(review) === 0"
      >
        <summary class="review-summary">
          <span class="review-date">{{ formatDate(review.created_at) }}</span>
          <SBadge variant="info">{{ review.worktree_branch }}</SBadge>
        </summary>
        <div class="review-content">
          <MarkdownRenderer :content="review.review_text" />
        </div>
      </details>
    </div>

    <SEmptyState
      v-else-if="!reviewing && !error"
      title="No AI reviews yet"
      description="Click &quot;Run Review&quot; to generate one."
    />
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

.review-loading {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--space-3);
  padding: var(--space-6);
}

.loading-text {
  font-size: 13px;
  color: var(--color-text-muted);
  text-align: center;
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

.review-content {
  padding: var(--space-4);
  border-top: 1px solid var(--color-border-default);
}

/* Review comparison toggle */
.comparison-toggle-wrapper {
  display: flex;
  justify-content: flex-end;
}
</style>
