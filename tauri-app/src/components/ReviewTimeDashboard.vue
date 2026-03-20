<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
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
  if (!stats.value) return '—'
  return formatDuration(stats.value.avg_review_seconds)
})

const totalFormatted = computed(() => {
  if (!stats.value) return '—'
  return formatDuration(stats.value.total_seconds)
})
</script>

<template>
  <div class="review-time-dashboard">
    <h2 class="section-title">Review Time Insights</h2>

    <div v-if="loading" class="loading-state">Loading review time data...</div>
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
        <h3 class="subsection-title">Average by Risk Tier</h3>
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
        <h3 class="subsection-title">Weekly Trend</h3>
        <div class="trend-rows">
          <div v-for="week in stats.weekly_trend" :key="week.week_start" class="trend-row">
            <span class="trend-week">{{ week.week_start }}</span>
            <span class="trend-reviews">{{ week.review_count }} reviews</span>
            <span class="trend-time">{{ formatDuration(week.total_seconds) }}</span>
          </div>
        </div>
      </div>
    </div>
    <div v-else class="empty-state">
      No review time data yet. Time is tracked automatically when you open PR detail views.
    </div>
  </div>
</template>

<style scoped>
.review-time-dashboard {
  background: var(--color-surface-panel);
  border: 1px solid var(--color-border-default);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-card);
  padding: var(--space-5);
}

.section-title {
  font-size: var(--text-subheading-size);
  font-weight: var(--text-subheading-weight);
  color: var(--color-text-primary);
  margin-bottom: var(--space-4);
}

.subsection-title {
  font-size: 12px;
  font-weight: 600;
  color: var(--color-text-muted);
  text-transform: uppercase;
  letter-spacing: 0.05em;
  margin-bottom: var(--space-3);
  margin-top: var(--space-4);
}

.loading-state, .error-state, .empty-state {
  text-align: center;
  color: var(--color-text-muted);
  font-size: 13px;
  padding: var(--space-4);
}

.time-stats-row {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: var(--space-4);
}

.time-stat {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--space-1);
}

.time-stat-value {
  font-size: 24px;
  font-weight: 700;
  font-family: var(--font-mono);
  color: var(--color-text-primary);
}

.time-stat-label {
  font-size: 12px;
  color: var(--color-text-muted);
}

.tier-bars {
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
}

.tier-bar-row {
  display: flex;
  align-items: center;
  gap: var(--space-2);
}

.tier-label {
  font-size: 12px;
  font-weight: 600;
  width: 56px;
  text-transform: capitalize;
}

.tier-low { color: var(--color-status-success); }
.tier-medium { color: var(--color-status-warning); }
.tier-high { color: var(--color-status-danger); }

.tier-bar-track {
  flex: 1;
  height: 8px;
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
  font-size: 12px;
  color: var(--color-text-secondary);
  width: 48px;
  text-align: right;
}

.tier-count {
  font-size: 11px;
  color: var(--color-text-muted);
}

.trend-rows {
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
}

.trend-row {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  font-size: 12px;
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
