<script setup lang="ts">
import { watch, computed } from 'vue'
import { SSelect, SBadge, SListRow, SSectionHeader, SEmptyState } from '@stuntrocket/ui'
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

/** String-based wrappers for SSelect compatibility */
const olderReviewIdStr = computed({
  get: () => olderReviewId.value != null ? String(olderReviewId.value) : '',
  set: (val: string) => { olderReviewId.value = val ? Number(val) : null },
})

const newerReviewIdStr = computed({
  get: () => newerReviewId.value != null ? String(newerReviewId.value) : '',
  set: (val: string) => { newerReviewId.value = val ? Number(val) : null },
})

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

function statusVariant(issue: ComparedIssue): 'success' | 'warning' | 'default' {
  if (issue.status === 'resolved') return 'success'
  if (issue.status === 'new') return 'warning'
  return 'default'
}
</script>

<template>
  <div class="ai-comparison">
    <div class="comparison-header">
      <SSectionHeader title="Review Comparison" />
      <div class="comparison-selectors">
        <div class="selector-group">
          <span class="selector-label">Older</span>
          <SSelect v-model="olderReviewIdStr">
            <option
              v-for="review in reviews"
              :key="review.id"
              :value="String(review.id)"
            >
              {{ formatDate(review.created_at) }}
            </option>
          </SSelect>
        </div>
        <span class="selector-arrow">vs</span>
        <div class="selector-group">
          <span class="selector-label">Newer</span>
          <SSelect v-model="newerReviewIdStr">
            <option
              v-for="review in reviews"
              :key="review.id"
              :value="String(review.id)"
            >
              {{ formatDate(review.created_at) }}
            </option>
          </SSelect>
        </div>
      </div>
    </div>

    <div v-if="hasComparison" class="comparison-body">
      <!-- Summary counters -->
      <div class="comparison-summary">
        <div class="summary-stat resolved">
          <SBadge variant="success">{{ totalResolved }}</SBadge>
          <span class="summary-label">Resolved</span>
        </div>
        <div class="summary-stat new-issues">
          <SBadge variant="warning">{{ totalNew }}</SBadge>
          <span class="summary-label">New</span>
        </div>
        <div class="summary-stat persistent">
          <SBadge variant="default">{{ totalPersistent }}</SBadge>
          <span class="summary-label">Persistent</span>
        </div>
      </div>

      <!-- Resolved issues -->
      <div v-if="comparison!.resolvedIssues.length > 0" class="issue-group">
        <h4 class="group-heading resolved-heading">Resolved Issues</h4>
        <SListRow
          v-for="(issue, i) in comparison!.resolvedIssues"
          :key="`resolved-${i}`"
        >
          <template #default>
            <div class="issue-row-content">
              <SBadge :variant="statusVariant(issue)">{{ statusIcon(issue) }}</SBadge>
              <span class="issue-severity" :class="`severity-${issue.severity}`">{{ severityIcon(issue.severity) }}</span>
              <code v-if="issue.file" class="issue-file">{{ issue.file }}</code>
              <span class="issue-description">{{ issue.description }}</span>
            </div>
          </template>
        </SListRow>
      </div>

      <!-- New issues -->
      <div v-if="comparison!.newIssues.length > 0" class="issue-group">
        <h4 class="group-heading new-heading">New Issues</h4>
        <SListRow
          v-for="(issue, i) in comparison!.newIssues"
          :key="`new-${i}`"
        >
          <template #default>
            <div class="issue-row-content">
              <SBadge :variant="statusVariant(issue)">{{ statusIcon(issue) }}</SBadge>
              <span class="issue-severity" :class="`severity-${issue.severity}`">{{ severityIcon(issue.severity) }}</span>
              <code v-if="issue.file" class="issue-file">{{ issue.file }}</code>
              <span class="issue-description">{{ issue.description }}</span>
            </div>
          </template>
        </SListRow>
      </div>

      <!-- Persistent issues -->
      <div v-if="comparison!.persistentIssues.length > 0" class="issue-group">
        <h4 class="group-heading persistent-heading">Persistent Issues</h4>
        <SListRow
          v-for="(issue, i) in comparison!.persistentIssues"
          :key="`persistent-${i}`"
        >
          <template #default>
            <div class="issue-row-content">
              <SBadge :variant="statusVariant(issue)">{{ statusIcon(issue) }}</SBadge>
              <span class="issue-severity" :class="`severity-${issue.severity}`">{{ severityIcon(issue.severity) }}</span>
              <code v-if="issue.file" class="issue-file">{{ issue.file }}</code>
              <span class="issue-description">{{ issue.description }}</span>
            </div>
          </template>
        </SListRow>
      </div>

      <!-- Empty state when no issues found in either review -->
      <SEmptyState
        v-if="totalNew === 0 && totalResolved === 0 && totalPersistent === 0"
        title="No structured issues found"
        description="The reviews may not follow the expected format."
      />
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

.summary-label {
  font-size: 11px;
  font-weight: 500;
  color: var(--color-text-muted);
  text-transform: uppercase;
  letter-spacing: 0.03em;
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

.issue-row-content {
  display: flex;
  align-items: flex-start;
  gap: var(--space-2);
  font-size: 12px;
  line-height: 1.4;
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
</style>
