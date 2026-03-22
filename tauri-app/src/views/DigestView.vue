<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useReviewDigest } from '../composables/useReviewDigest'
import { SCard, SEmptyState, SSectionHeader } from '@stuntrocket/ui'
import ContentLoader from '../components/ContentLoader.vue'

const { digest, loading, error, fetchDigest } = useReviewDigest()
const selectedPeriod = ref(7)
const periodOptions = [
  { value: 7, label: '7 days' },
  { value: 14, label: '14 days' },
  { value: 30, label: '30 days' },
]

onMounted(() => {
  fetchDigest(selectedPeriod.value)
})

async function changePeriod(days: number) {
  selectedPeriod.value = days
  await fetchDigest(days)
}

function formatDuration(seconds: number): string {
  if (seconds === 0) return '—'
  const mins = Math.floor(seconds / 60)
  if (mins < 60) return `${mins}m`
  const hours = Math.floor(mins / 60)
  const remainMins = mins % 60
  return `${hours}h ${remainMins}m`
}

function formatDate(dateStr: string): string {
  return new Date(dateStr).toLocaleDateString('en-GB', {
    day: 'numeric',
    month: 'short',
  })
}

/** Compute percentage change between current and previous values. */
function pctChange(current: number, previous: number | undefined): string {
  if (previous == null || previous === 0) {
    return current > 0 ? '+' : '—'
  }
  const pct = ((current - previous) / previous) * 100
  const sign = pct > 0 ? '+' : ''
  return `${sign}${Math.round(pct)}%`
}

/** Determine trend direction for styling. */
function trendClass(current: number, previous: number | undefined, invertIsGood = false): string {
  if (previous == null) return ''
  if (current > previous) return invertIsGood ? 'trend-down' : 'trend-up'
  if (current < previous) return invertIsGood ? 'trend-up' : 'trend-down'
  return 'trend-flat'
}

const periodLabel = computed(() => {
  if (!digest.value) return ''
  return `${formatDate(digest.value.period_start)} — ${formatDate(digest.value.period_end)}`
})
</script>

<template>
  <div class="digest-view">
    <div class="digest-header">
      <SSectionHeader title="Review Digest" />
      <div class="period-selector">
        <button
          v-for="option in periodOptions"
          :key="option.value"
          class="period-btn"
          :class="{ active: selectedPeriod === option.value }"
          @click="changePeriod(option.value)"
        >
          {{ option.label }}
        </button>
      </div>
    </div>

    <p v-if="digest" class="period-range">{{ periodLabel }}</p>

    <ContentLoader v-if="loading" variant="cards" :count="4" />

    <div v-else-if="error" class="digest-error">{{ error }}</div>

    <div v-else-if="digest" class="digest-grid">
      <!-- Reviewed -->
      <SCard variant="content" class="digest-card">
        <div class="card-header">
          <span class="card-label">Reviewed</span>
          <span
            v-if="digest.previous"
            class="card-trend"
            :class="trendClass(digest.reviewed_count, digest.previous.reviewed_count)"
          >
            {{ pctChange(digest.reviewed_count, digest.previous.reviewed_count) }}
          </span>
        </div>
        <span class="card-value">{{ digest.reviewed_count }}</span>
        <span v-if="digest.previous" class="card-previous">
          prev: {{ digest.previous.reviewed_count }}
        </span>
      </SCard>

      <!-- Merged -->
      <SCard variant="content" class="digest-card">
        <div class="card-header">
          <span class="card-label">Merged</span>
          <span
            v-if="digest.previous"
            class="card-trend"
            :class="trendClass(digest.merged_count, digest.previous.merged_count)"
          >
            {{ pctChange(digest.merged_count, digest.previous.merged_count) }}
          </span>
        </div>
        <span class="card-value merged">{{ digest.merged_count }}</span>
        <span v-if="digest.previous" class="card-previous">
          prev: {{ digest.previous.merged_count }}
        </span>
      </SCard>

      <!-- Pending -->
      <SCard variant="content" class="digest-card">
        <div class="card-header">
          <span class="card-label">Pending</span>
          <span
            v-if="digest.previous"
            class="card-trend"
            :class="trendClass(digest.pending_count, digest.previous.pending_count, true)"
          >
            {{ pctChange(digest.pending_count, digest.previous.pending_count) }}
          </span>
        </div>
        <span class="card-value pending">{{ digest.pending_count }}</span>
        <span v-if="digest.previous" class="card-previous">
          prev: {{ digest.previous.pending_count }}
        </span>
      </SCard>

      <!-- Avg Review Time -->
      <SCard variant="content" class="digest-card">
        <div class="card-header">
          <span class="card-label">Avg Review Time</span>
          <span
            v-if="digest.previous"
            class="card-trend"
            :class="trendClass(digest.avg_review_seconds, digest.previous.avg_review_seconds, true)"
          >
            {{ pctChange(digest.avg_review_seconds, digest.previous.avg_review_seconds) }}
          </span>
        </div>
        <span class="card-value">{{ formatDuration(digest.avg_review_seconds) }}</span>
        <span v-if="digest.previous" class="card-previous">
          prev: {{ formatDuration(digest.previous.avg_review_seconds) }}
        </span>
      </SCard>

      <!-- Stale -->
      <SCard variant="content" class="digest-card">
        <div class="card-header">
          <span class="card-label">Stale PRs</span>
          <span
            v-if="digest.previous"
            class="card-trend"
            :class="trendClass(digest.stale_count, digest.previous.stale_count, true)"
          >
            {{ pctChange(digest.stale_count, digest.previous.stale_count) }}
          </span>
        </div>
        <span class="card-value stale">{{ digest.stale_count }}</span>
        <span v-if="digest.previous" class="card-previous">
          prev: {{ digest.previous.stale_count }}
        </span>
      </SCard>

      <!-- Total Open -->
      <SCard variant="content" class="digest-card">
        <div class="card-header">
          <span class="card-label">Total Open</span>
        </div>
        <span class="card-value">{{ digest.total_open }}</span>
      </SCard>
    </div>

    <SEmptyState
      v-else
      title="No data available"
      description="Sync your repositories first."
    />
  </div>
</template>

<style scoped>
.digest-view {
  width: 100%;
}

.digest-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: var(--space-2);
}


.period-selector {
  display: flex;
  gap: var(--space-1);
  background: var(--color-surface-raised);
  border: 1px solid var(--color-border-default);
  border-radius: var(--radius-md);
  padding: 2px;
}

.period-btn {
  padding: var(--space-1) var(--space-3);
  font-size: 13px;
  font-weight: 500;
  border: none;
  border-radius: var(--radius-sm);
  background: none;
  color: var(--color-text-muted);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.period-btn:hover {
  color: var(--color-text-primary);
  background: var(--color-surface-hover);
}

.period-btn.active {
  background: var(--color-accent-muted);
  color: var(--color-accent);
  font-weight: 600;
}

.period-range {
  font-size: 13px;
  color: var(--color-text-muted);
  margin-bottom: var(--space-6);
}


.digest-error {
  text-align: center;
  color: var(--color-status-danger);
  padding: var(--space-12);
  font-size: 14px;
}

.digest-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: var(--space-4);
}

.digest-card {
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.card-label {
  font-size: 12px;
  font-weight: 600;
  color: var(--color-text-muted);
  text-transform: uppercase;
  letter-spacing: 0.03em;
}

.card-trend {
  font-size: 12px;
  font-weight: 600;
  font-family: var(--font-mono);
  padding: 1px var(--space-2);
  border-radius: var(--radius-full);
}

.card-trend.trend-up {
  background: rgba(34, 197, 94, 0.15);
  color: var(--color-status-success);
}

.card-trend.trend-down {
  background: rgba(220, 38, 38, 0.15);
  color: var(--color-status-danger);
}

.card-trend.trend-flat {
  background: rgba(100, 116, 139, 0.15);
  color: var(--color-text-muted);
}

.card-value {
  font-size: 36px;
  font-weight: 700;
  font-family: var(--font-mono);
  color: var(--color-text-primary);
  line-height: 1;
}

.card-value.merged {
  color: #a78bfa;
}

.card-value.pending {
  color: var(--color-status-warning);
}

.card-value.stale {
  color: var(--color-status-danger);
}

.card-previous {
  font-size: 12px;
  color: var(--color-text-muted);
  font-family: var(--font-mono);
}
</style>
