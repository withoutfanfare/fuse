<script setup lang="ts">
import { computed } from 'vue'
import type { ReviewerWorkloadStats } from '../types'
import ContentLoader from './ContentLoader.vue'

const props = withDefaults(defineProps<{
  workload: ReviewerWorkloadStats[]
  loading?: boolean
  error?: string | null
}>(), {
  loading: false,
  error: null,
})

const emit = defineEmits<{
  refresh: []
}>()

/** Maximum total count across all reviewers, used to scale bar widths. */
const maxTotal = computed(() => {
  if (props.workload.length === 0) return 1
  return Math.max(
    ...props.workload.map(r => r.assigned_count + r.completed_count + r.overdue_count),
    1,
  )
})

/** Format average response time for display. */
function formatResponseTime(hours: number): string {
  if (hours === 0) return '--'
  if (hours < 1) return `${Math.round(hours * 60)}m`
  if (hours < 24) return `${hours.toFixed(1)}h`
  return `${(hours / 24).toFixed(1)}d`
}

/** Compute percentage width for a bar segment. */
function barWidth(count: number): string {
  if (maxTotal.value === 0) return '0%'
  return `${(count / maxTotal.value) * 100}%`
}
</script>

<template>
  <div class="workload-dashboard">
    <div class="workload-header">
      <h2 class="section-title">Reviewer Workload</h2>
      <button class="refresh-btn" @click="emit('refresh')" :disabled="loading">
        {{ loading ? 'Loading...' : 'Refresh' }}
      </button>
    </div>

    <p v-if="error" class="workload-error">{{ error }}</p>

    <ContentLoader v-if="loading" variant="list" :count="3" />

    <div v-else-if="workload.length > 0" class="workload-list">
      <div class="workload-legend">
        <span class="legend-item">
          <span class="legend-swatch legend-swatch--completed" />
          Completed
        </span>
        <span class="legend-item">
          <span class="legend-swatch legend-swatch--assigned" />
          Assigned
        </span>
        <span class="legend-item">
          <span class="legend-swatch legend-swatch--overdue" />
          Overdue
        </span>
      </div>

      <div
        v-for="reviewer in workload"
        :key="reviewer.reviewer"
        class="workload-row"
      >
        <div class="reviewer-info">
          <span class="reviewer-name">{{ reviewer.reviewer }}</span>
          <span class="reviewer-response">
            {{ formatResponseTime(reviewer.avg_response_hours) }} avg
          </span>
        </div>

        <div class="bar-container">
          <div class="bar-track">
            <div
              class="bar-segment bar-segment--completed"
              :style="{ width: barWidth(reviewer.completed_count) }"
              :title="`Completed: ${reviewer.completed_count}`"
            />
            <div
              class="bar-segment bar-segment--assigned"
              :style="{ width: barWidth(reviewer.assigned_count) }"
              :title="`Assigned: ${reviewer.assigned_count}`"
            />
            <div
              class="bar-segment bar-segment--overdue"
              :style="{ width: barWidth(reviewer.overdue_count) }"
              :title="`Overdue: ${reviewer.overdue_count}`"
            />
          </div>
          <div class="bar-counts">
            <span v-if="reviewer.completed_count > 0" class="count count--completed">
              {{ reviewer.completed_count }}
            </span>
            <span v-if="reviewer.assigned_count > 0" class="count count--assigned">
              {{ reviewer.assigned_count }}
            </span>
            <span v-if="reviewer.overdue_count > 0" class="count count--overdue">
              {{ reviewer.overdue_count }}
            </span>
          </div>
        </div>
      </div>
    </div>

    <div v-else-if="workload.length === 0" class="workload-empty">
      <p>No reviewer workload data yet. Sync your repositories to populate reviewer assignments.</p>
    </div>
  </div>
</template>

<style scoped>
.workload-dashboard {
  width: 100%;
}

.workload-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: var(--space-4);
}

.section-title {
  font-size: 16px;
  font-weight: 600;
  color: var(--color-text-primary);
}

.refresh-btn {
  font-size: 12px;
  padding: var(--space-1) var(--space-3);
  background: var(--color-surface-raised);
  border: 1px solid var(--color-border-default);
  border-radius: var(--radius-md);
  color: var(--color-text-secondary);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.refresh-btn:hover:not(:disabled) {
  background: var(--color-surface-hover);
  color: var(--color-text-primary);
}

.refresh-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.workload-error {
  color: var(--color-status-danger);
  font-size: 13px;
  margin-bottom: var(--space-3);
}

.workload-legend {
  display: flex;
  gap: var(--space-4);
  margin-bottom: var(--space-3);
  justify-content: flex-end;
}

.legend-item {
  display: flex;
  align-items: center;
  gap: var(--space-1);
  font-size: 11px;
  color: var(--color-text-muted);
}

.legend-swatch {
  width: 10px;
  height: 10px;
  border-radius: var(--radius-sm);
  display: inline-block;
}

.legend-swatch--completed {
  background: var(--color-status-success);
}

.legend-swatch--assigned {
  background: var(--color-status-warning);
}

.legend-swatch--overdue {
  background: var(--color-status-danger);
}

.workload-list {
  display: flex;
  flex-direction: column;
  gap: var(--space-3);
}

.workload-row {
  display: flex;
  align-items: center;
  gap: var(--space-3);
}

.reviewer-info {
  min-width: 140px;
  max-width: 140px;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.reviewer-name {
  font-size: 13px;
  font-weight: 500;
  color: var(--color-text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.reviewer-response {
  font-size: 11px;
  color: var(--color-text-muted);
  font-family: var(--font-mono);
}

.bar-container {
  flex: 1;
  display: flex;
  align-items: center;
  gap: var(--space-2);
}

.bar-track {
  flex: 1;
  display: flex;
  height: 20px;
  background: rgba(255, 255, 255, 0.04);
  border-radius: var(--radius-sm);
  overflow: hidden;
}

.bar-segment {
  height: 100%;
  transition: width 0.4s ease;
  min-width: 0;
}

.bar-segment--completed {
  background: var(--color-status-success);
  opacity: 0.85;
}

.bar-segment--assigned {
  background: var(--color-status-warning);
  opacity: 0.85;
}

.bar-segment--overdue {
  background: var(--color-status-danger);
  opacity: 0.85;
}

.bar-counts {
  display: flex;
  gap: var(--space-1);
  min-width: 80px;
}

.count {
  font-size: 11px;
  font-weight: 600;
  font-family: var(--font-mono);
  padding: 1px var(--space-1);
  border-radius: var(--radius-sm);
}

.count--completed {
  color: var(--color-status-success);
}

.count--assigned {
  color: var(--color-status-warning);
}

.count--overdue {
  color: var(--color-status-danger);
}

.workload-empty {
  text-align: center;
  padding: var(--space-6);
  color: var(--color-text-muted);
  font-size: 13px;
}
</style>
