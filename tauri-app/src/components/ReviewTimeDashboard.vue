<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { SCard, SSectionHeader, SSpinner, SEmptyState } from '@stuntrocket/ui'
import type { ReviewVelocityStats } from '../types'

const loading = ref(true)
const error = ref<string | null>(null)
const stats = ref<ReviewVelocityStats | null>(null)

onMounted(async () => {
  try {
    stats.value = await invoke<ReviewVelocityStats>('get_review_velocity_stats')
  } catch (e) {
    error.value = String(e)
  } finally {
    loading.value = false
  }
})

function formatDuration(seconds: number): string {
  if (seconds < 60) return `${Math.round(seconds)}s`
  const mins = Math.floor(seconds / 60)
  if (mins < 60) return `${mins}m`
  const hrs = Math.floor(mins / 60)
  const remainMins = mins % 60
  return `${hrs}h ${remainMins}m`
}

const avgFormatted = computed(() => {
  if (!stats.value) return '--'
  return formatDuration(stats.value.avg_review_seconds)
})

const totalFormatted = computed(() => {
  if (!stats.value) return '--'
  return formatDuration(stats.value.total_seconds)
})
</script>

<template>
  <SCard variant="content">
    <SSectionHeader title="Review Time Insights" />

    <div v-if="loading" class="loading-state">
      <SSpinner />
      <span>Loading review time data...</span>
    </div>
    <div v-else-if="error" class="error-state">{{ error }}</div>
    <div v-else-if="stats && stats.total_reviews > 0" class="time-content">
      <div class="time-stats-row">
        <div class="time-stat">
          <span class="time-stat-value">{{ avgFormatted }}</span>
          <span class="time-stat-label">Avg Review Time</span>
        </div>
        <div class="time-stat">
          <span class="time-stat-value">{{ stats.total_reviews }}</span>
          <span class="time-stat-label">Total Reviews</span>
        </div>
        <div class="time-stat">
          <span class="time-stat-value">{{ totalFormatted }}</span>
          <span class="time-stat-label">Total Time</span>
        </div>
      </div>

      <div v-if="stats.by_risk_tier.length > 0" class="time-breakdown">
        <SSectionHeader title="Average by Risk Tier" />
        <div class="tier-bars">
          <div v-for="tier in stats.by_risk_tier" :key="tier.tier" class="tier-bar-row">
            <span class="tier-label" :class="`tier-${tier.tier}`">{{ tier.tier }}</span>
            <div class="tier-bar-track">
              <div
                class="tier-bar-fill"
                :class="`tier-${tier.tier}`"
                :style="{ width: `${Math.min(100, (tier.avg_seconds / (stats!.avg_review_seconds * 2)) * 100)}%` }"
              />
            </div>
            <span class="tier-value">{{ formatDuration(tier.avg_seconds) }}</span>
            <span class="tier-count">({{ tier.count }})</span>
          </div>
        </div>
      </div>

      <div v-if="stats.weekly_trend.length > 0" class="time-trend">
        <SSectionHeader title="Weekly Trend" />
        <div class="trend-rows">
          <div v-for="week in stats.weekly_trend" :key="week.week_start" class="trend-row">
            <span class="trend-week">{{ week.week_start }}</span>
            <span class="trend-reviews">{{ week.review_count }} reviews</span>
            <span class="trend-time">{{ formatDuration(week.total_seconds) }}</span>
          </div>
        </div>
      </div>
    </div>
    <SEmptyState
      v-else
      title="No review time data yet"
      description="Time is tracked automatically when you open PR detail views."
    />
  </SCard>
</template>

<style scoped>
:deep(.py-12) {
  padding-top: var(--space-4) !important;
  padding-bottom: var(--space-4) !important;
}

.loading-state {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  justify-content: center;
  color: var(--color-text-muted);
  font-size: 12px;
  padding: var(--space-2);
}

.error-state {
  text-align: center;
  color: var(--color-text-muted);
  font-size: 12px;
  padding: var(--space-2);
}

.time-content {
  margin-top: var(--space-2);
}

.time-stats-row {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: var(--space-2);
}

.time-stat {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--space-1);
}

.time-stat-value {
  font-size: 15px;
  font-weight: 700;
  font-family: var(--font-mono);
  color: var(--color-text-primary);
}

.time-stat-label {
  font-size: 11px;
  color: var(--color-text-muted);
}

.time-breakdown {
  margin-top: var(--space-2);
}

.tier-bars {
  display: flex;
  flex-direction: column;
  gap: var(--space-1);
  margin-top: var(--space-2);
}

.tier-bar-row {
  display: flex;
  align-items: center;
  gap: var(--space-2);
}

.tier-label {
  font-size: 11px;
  font-weight: 600;
  width: 50px;
  text-transform: capitalize;
}

.tier-low { color: var(--color-status-success); }
.tier-medium { color: var(--color-status-warning); }
.tier-high { color: var(--color-status-danger); }

.tier-bar-track {
  flex: 1;
  height: 6px;
  background: rgba(255, 255, 255, 0.05);
  border-radius: var(--radius-full);
  overflow: hidden;
}

.tier-bar-fill {
  height: 100%;
  border-radius: var(--radius-full);
  transition: width 0.5s ease;
}

.tier-bar-fill.tier-low { background: var(--color-status-success); }
.tier-bar-fill.tier-medium { background: var(--color-status-warning); }
.tier-bar-fill.tier-high { background: var(--color-status-danger); }

.tier-value {
  font-family: var(--font-mono);
  font-size: 11px;
  color: var(--color-text-secondary);
  width: 44px;
  text-align: right;
}

.tier-count {
  font-size: 11px;
  color: var(--color-text-muted);
}

.time-trend {
  margin-top: var(--space-2);
}

.trend-rows {
  display: flex;
  flex-direction: column;
  gap: var(--space-1);
  margin-top: var(--space-2);
}

.trend-row {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  font-size: 11px;
  padding: var(--space-1) 0;
  border-bottom: 1px solid var(--color-border-default);
}

.trend-row:last-child {
  border-bottom: none;
}

.trend-week {
  font-family: var(--font-mono);
  color: var(--color-text-muted);
  width: 80px;
}

.trend-reviews {
  color: var(--color-text-secondary);
  flex: 1;
}

.trend-time {
  font-family: var(--font-mono);
  color: var(--color-text-primary);
  font-weight: 600;
}
</style>
