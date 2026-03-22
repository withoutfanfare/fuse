<script setup lang="ts">
import { computed, toRef } from 'vue'
import { SKpiCard, SSparkline, useCountUp } from '@stuntrocket/ui'

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

/** Map our 'neutral' variant to SKpiCard's 'default' */
const kpiVariant = computed(() =>
  props.variant === 'neutral' || !props.variant ? 'default' : props.variant,
)
</script>

<template>
  <div class="stats-card">
    <SKpiCard :label="label" :value="renderedValue" :variant="kpiVariant">
      <template v-if="history && history.length >= 2" #footer>
        <SSparkline
          :values="history"
          :gradient="true"
          class="stats-sparkline"
        />
      </template>
    </SKpiCard>
  </div>
</template>

<style scoped>
.stats-card {
  min-width: 0;
}

/* Override SKpiCard's default border/bg to match the glassmorphic design system */
.stats-card :deep(> div) {
  background: var(--color-surface-panel);
  border-color: var(--color-border-default);
  box-shadow: var(--shadow-card);
  transition: transform var(--transition-fast), box-shadow var(--transition-fast);
}

.stats-card:hover :deep(> div) {
  transform: translateY(-1px);
  box-shadow: var(--shadow-card-hover, var(--shadow-card));
}

.stats-sparkline {
  margin-top: var(--space-1);
}
</style>
