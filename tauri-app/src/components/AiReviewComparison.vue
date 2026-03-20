<script setup lang="ts">
import { watch, computed } from 'vue'
import { useAiReviewComparison } from '../composables/useAiReviewComparison'
import type { AiReview, ComparedIssue, AiReviewIssueSeverity } from '../types'

const props = defineProps<{
  reviews: AiReview[]
}>()

const {
  olderReviewId,
  newerReviewId,
  comparison,
  hasComparison,
  compareSelectedReviews,
} = useAiReviewComparison()

// Auto-select the two most recent reviews when reviews change
watch(
  () => props.reviews,
  (reviews) => {
    if (reviews.length >= 2) {
      // Reviews are sorted newest-first from the backend
      newerReviewId.value = reviews[0].id
      olderReviewId.value = reviews[1].id
      compareSelectedReviews(reviews)
    }
  },
  { immediate: true }
)

// Re-compare when selection changes
watch([olderReviewId, newerReviewId], () => {
  compareSelectedReviews(props.reviews)
})

const totalNew = computed(() => comparison.value?.newIssues.length ?? 0)
const totalResolved = computed(() => comparison.value?.resolvedIssues.length ?? 0)
const totalPersistent = computed(() => comparison.value?.persistentIssues.length ?? 0)

function formatDate(dateStr: string): string {
  return new Date(dateStr).toLocaleDateString('en-GB', {
    day: 'numeric', month: 'short',
    hour: '2-digit', minute: '2-digit',
  })
}

function severityIcon(severity: AiReviewIssueSeverity): string {
  if (severity === 'critical') return '!!'
  if (severity === 'warning') return '!'
  return '~'
}

function statusIcon(issue: ComparedIssue): string {
  if (issue.status === 'new') return '+'
  if (issue.status === 'resolved') return '-'
  return '='
}
</script>

<template>
  <div class="ai-comparison">
    <div class="comparison-header">
      <h3 class="comparison-title">Review Comparison</h3>
      <div class="comparison-selectors">
        <label class="selector-group">
          <span class="selector-label">Older</span>
          <select v-model="olderReviewId" class="selector-input">
            <option
              v-for="review in reviews"
              :key="review.id"
              :value="review.id"
            >
              {{ formatDate(review.created_at) }}
            </option>
          </select>
        </label>
        <span class="selector-arrow">vs</span>
        <label class="selector-group">
          <span class="selector-label">Newer</span>
          <select v-model="newerReviewId" class="selector-input">
            <option
              v-for="review in reviews"
              :key="review.id"
              :value="review.id"
            >
              {{ formatDate(review.created_at) }}
            </option>
          </select>
        </label>
      </div>
    </div>

    <div v-if="hasComparison" class="comparison-body">
      <!-- Summary counters -->
      <div class="comparison-summary">
        <div class="summary-stat resolved">
          <span class="summary-count">{{ totalResolved }}</span>
          <span class="summary-label">Resolved</span>
        </div>
        <div class="summary-stat new-issues">
          <span class="summary-count">{{ totalNew }}</span>
          <span class="summary-label">New</span>
        </div>
        <div class="summary-stat persistent">
          <span class="summary-count">{{ totalPersistent }}</span>
          <span class="summary-label">Persistent</span>
        </div>
      </div>

      <!-- Resolved issues -->
      <div v-if="comparison!.resolvedIssues.length > 0" class="issue-group">
        <h4 class="group-heading resolved-heading">Resolved Issues</h4>
        <div
          v-for="(issue, i) in comparison!.resolvedIssues"
          :key="`resolved-${i}`"
          class="issue-row resolved-row"
        >
          <span class="issue-status-badge resolved-badge">{{ statusIcon(issue) }}</span>
          <span class="issue-severity" :class="`severity-${issue.severity}`">{{ severityIcon(issue.severity) }}</span>
          <code v-if="issue.file" class="issue-file">{{ issue.file }}</code>
          <span class="issue-description">{{ issue.description }}</span>
        </div>
      </div>

      <!-- New issues -->
      <div v-if="comparison!.newIssues.length > 0" class="issue-group">
        <h4 class="group-heading new-heading">New Issues</h4>
        <div
          v-for="(issue, i) in comparison!.newIssues"
          :key="`new-${i}`"
          class="issue-row new-row"
        >
          <span class="issue-status-badge new-badge">{{ statusIcon(issue) }}</span>
          <span class="issue-severity" :class="`severity-${issue.severity}`">{{ severityIcon(issue.severity) }}</span>
          <code v-if="issue.file" class="issue-file">{{ issue.file }}</code>
          <span class="issue-description">{{ issue.description }}</span>
        </div>
      </div>

      <!-- Persistent issues -->
      <div v-if="comparison!.persistentIssues.length > 0" class="issue-group">
        <h4 class="group-heading persistent-heading">Persistent Issues</h4>
        <div
          v-for="(issue, i) in comparison!.persistentIssues"
          :key="`persistent-${i}`"
          class="issue-row persistent-row"
        >
          <span class="issue-status-badge persistent-badge">{{ statusIcon(issue) }}</span>
          <span class="issue-severity" :class="`severity-${issue.severity}`">{{ severityIcon(issue.severity) }}</span>
          <code v-if="issue.file" class="issue-file">{{ issue.file }}</code>
          <span class="issue-description">{{ issue.description }}</span>
        </div>
      </div>

      <!-- Empty state when no issues found in either review -->
      <div
        v-if="totalNew === 0 && totalResolved === 0 && totalPersistent === 0"
        class="comparison-empty"
      >
        No structured issues found in either review. The reviews may not follow the expected format.
      </div>
    </div>
  </div>
</template>

<style scoped>
.ai-comparison {
  display: flex;
  flex-direction: column;
  gap: var(--space-4);
}

.comparison-header {
  display: flex;
  flex-direction: column;
  gap: var(--space-3);
}

.comparison-title {
  font-size: 13px;
  font-weight: 600;
  color: var(--color-text-secondary);
  margin: 0;
}

.comparison-selectors {
  display: flex;
  align-items: flex-end;
  gap: var(--space-3);
}

.selector-group {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.selector-label {
  font-size: 11px;
  font-weight: 500;
  color: var(--color-text-muted);
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.selector-input {
  background: var(--color-surface-input);
  border: 1px solid var(--color-border-default);
  border-radius: var(--radius-sm);
  color: var(--color-text-primary);
  font-size: 12px;
  font-family: var(--font-mono);
  padding: var(--space-1) var(--space-2);
  cursor: pointer;
  transition: border-color var(--transition-fast);
}

.selector-input:focus {
  border-color: var(--color-border-focus);
  box-shadow: 0 0 0 2px var(--color-accent-muted);
  outline: none;
}

.selector-arrow {
  font-size: 12px;
  font-weight: 600;
  color: var(--color-text-muted);
  padding-bottom: 4px;
}

/* Summary counters */
.comparison-summary {
  display: flex;
  gap: var(--space-4);
  padding: var(--space-3);
  background: var(--color-surface-raised);
  border-radius: var(--radius-md);
  border: 1px solid var(--color-border-default);
}

.summary-stat {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 2px;
  flex: 1;
}

.summary-count {
  font-size: 20px;
  font-weight: 700;
  font-family: var(--font-mono);
}

.summary-label {
  font-size: 11px;
  font-weight: 500;
  color: var(--color-text-muted);
  text-transform: uppercase;
  letter-spacing: 0.03em;
}

.summary-stat.resolved .summary-count {
  color: var(--color-status-success);
}

.summary-stat.new-issues .summary-count {
  color: var(--color-status-warning);
}

.summary-stat.persistent .summary-count {
  color: var(--color-text-muted);
}

/* Issue groups */
.issue-group {
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
}

.group-heading {
  font-size: 12px;
  font-weight: 600;
  margin: 0;
  padding-bottom: var(--space-1);
  border-bottom: 1px solid var(--color-border-default);
}

.resolved-heading { color: var(--color-status-success); }
.new-heading { color: var(--color-status-warning); }
.persistent-heading { color: var(--color-text-muted); }

.issue-row {
  display: flex;
  align-items: flex-start;
  gap: var(--space-2);
  padding: var(--space-2) var(--space-3);
  border-radius: var(--radius-sm);
  font-size: 12px;
  line-height: 1.4;
}

.resolved-row {
  background: rgba(34, 197, 94, 0.06);
}

.new-row {
  background: rgba(234, 179, 8, 0.06);
}

.persistent-row {
  background: rgba(100, 116, 139, 0.06);
}

.issue-status-badge {
  flex-shrink: 0;
  width: 18px;
  height: 18px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border-radius: var(--radius-sm);
  font-size: 11px;
  font-weight: 700;
  font-family: var(--font-mono);
}

.resolved-badge {
  background: rgba(34, 197, 94, 0.2);
  color: var(--color-status-success);
}

.new-badge {
  background: rgba(234, 179, 8, 0.2);
  color: var(--color-status-warning);
}

.persistent-badge {
  background: rgba(100, 116, 139, 0.2);
  color: var(--color-text-muted);
}

.issue-severity {
  flex-shrink: 0;
  font-size: 10px;
  font-weight: 700;
  font-family: var(--font-mono);
}

.severity-critical { color: var(--color-status-danger); }
.severity-warning { color: var(--color-status-warning); }
.severity-suggestion { color: var(--color-status-info); }

.issue-file {
  flex-shrink: 0;
  background: rgba(255, 255, 255, 0.05);
  padding: 0 var(--space-1);
  border-radius: 2px;
  font-size: 11px;
  color: var(--color-text-secondary);
  max-width: 200px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.issue-description {
  color: var(--color-text-secondary);
  flex: 1;
  min-width: 0;
}

.comparison-empty {
  text-align: center;
  color: var(--color-text-muted);
  font-size: 13px;
  padding: var(--space-4);
}
</style>
