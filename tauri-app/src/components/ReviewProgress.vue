<script setup lang="ts">
import { computed } from 'vue'
import { SCard } from '@stuntrocket/ui'

const props = defineProps<{
  reviewed: number
  total: number
}>()

const percentage = computed(() => {
  if (props.total === 0) return 0
  return Math.round((props.reviewed / props.total) * 100)
})

// SVG donut parameters
const size = 72
const strokeWidth = 7
const radius = (size - strokeWidth) / 2
const circumference = 2 * Math.PI * radius

const dashOffset = computed(() => {
  if (props.total === 0) return circumference
  return circumference - (circumference * props.reviewed) / props.total
})
</script>

<template>
  <SCard variant="content" class="review-progress">
    <div class="donut-wrap">
      <svg :width="size" :height="size" class="donut-svg">
        <!-- Track -->
        <circle
          :cx="size / 2"
          :cy="size / 2"
          :r="radius"
          fill="none"
          :stroke-width="strokeWidth"
          class="donut-track"
        />
        <!-- Filled portion -->
        <circle
          :cx="size / 2"
          :cy="size / 2"
          :r="radius"
          fill="none"
          :stroke-width="strokeWidth"
          :stroke-dasharray="circumference"
          :stroke-dashoffset="dashOffset"
          stroke-linecap="round"
          class="donut-fill"
          :style="{ transform: 'rotate(-90deg)', transformOrigin: '50% 50%' }"
        />
      </svg>
      <div class="donut-label">
        <span class="donut-percentage">{{ percentage }}%</span>
      </div>
    </div>
    <p class="progress-text">
      {{ reviewed }} of {{ total }} PR{{ total === 1 ? '' : 's' }} reviewed
    </p>
  </SCard>
</template>

<style scoped>
.review-progress {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--space-3);
}

.donut-wrap {
  position: relative;
  display: flex;
  align-items: center;
  justify-content: center;
}

.donut-svg {
  display: block;
}

.donut-track {
  stroke: var(--color-risk-gauge-track);
}

.donut-fill {
  stroke: var(--color-accent);
  transition: stroke-dashoffset 0.6s cubic-bezier(0.4, 0, 0.2, 1);
}

.donut-label {
  position: absolute;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
}

.donut-percentage {
  font-size: 16px;
  font-weight: 700;
  color: var(--color-text-primary);
  font-family: var(--font-mono);
}

.progress-text {
  font-size: 11px;
  color: var(--color-text-secondary);
  margin: 0;
}
</style>
