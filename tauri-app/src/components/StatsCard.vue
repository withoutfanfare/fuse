<script setup lang="ts">
import { computed, toRef } from 'vue'
import { SCard, SSparkline } from '@stuntrocket/ui'
import { useCountUp } from '@stuntrocket/ui'

const props = defineProps<{
  label: string
  value: number | string
  variant?: 'neutral' | 'success' | 'warning' | 'danger' | 'info'
  history?: number[]
}>()

/**
 * Numeric values animate via useCountUp; string values render as-is.
 * We derive a numeric ref so the composable can watch it reactively.
 */
const numericTarget = computed(() =>
  typeof props.value === 'number' ? props.value : 0,
)

const { displayValue } = useCountUp(toRef(numericTarget), { duration: 600 })

const renderedValue = computed(() =>
  typeof props.value === 'number' ? displayValue.value : props.value,
)
</script>

<template>
  <SCard variant="glass" hoverable class="stats-card" :class="[`variant-${variant ?? 'neutral'}`]">
    <div class="stats-value">{{ renderedValue }}</div>
    <div class="stats-label">{{ label }}</div>
    <SSparkline
      v-if="history && history.length >= 2"
      :values="history"
      :gradient="true"
      class="stats-sparkline"
    />
  </SCard>
</template>

<style scoped>
.stats-value {
  font-size: 28px;
  font-weight: var(--text-display-weight);
  letter-spacing: var(--text-display-tracking);
  line-height: 1.1;
  margin-bottom: var(--space-1);
}

.stats-label {
  font-size: var(--text-caption-size);
  font-weight: var(--text-caption-weight);
  letter-spacing: var(--text-caption-tracking);
  line-height: var(--text-caption-leading);
  color: var(--color-text-secondary);
}

.variant-success .stats-value {
  color: var(--color-status-success);
  text-shadow: 0 0 20px rgba(34, 197, 94, 0.3);
}
.variant-warning .stats-value {
  color: var(--color-status-warning);
  text-shadow: 0 0 20px rgba(234, 179, 8, 0.3);
}
.variant-danger .stats-value {
  color: var(--color-status-danger);
  text-shadow: 0 0 20px rgba(220, 38, 38, 0.3);
}
.variant-info .stats-value {
  color: var(--color-status-info);
  text-shadow: 0 0 20px rgba(59, 130, 246, 0.3);
}
.variant-neutral .stats-value {
  color: var(--color-text-primary);
}

.stats-sparkline {
  margin-top: var(--space-2);
}
</style>
