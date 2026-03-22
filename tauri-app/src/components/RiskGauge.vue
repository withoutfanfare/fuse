<script setup lang="ts">
import { computed } from 'vue'
import { SGauge } from '@stuntrocket/ui'
import { riskLevel } from '../composables/useRiskScore'

const props = withDefaults(defineProps<{
  score: number
  size?: number
}>(), {
  size: 36,
})

/** Map 0-10 score to 0-100 percentage for SGauge's arc fill */
const gaugeValue = computed(() => Math.min(100, Math.max(0, props.score * 10)))

/**
 * Map risk level to SGauge variant (risk semantics are inverted: high = bad).
 * 1-3 low risk  → success (green)
 * 4-6 medium    → warning (yellow/orange)
 * 7-8 high      → danger  (red)
 * 9-10 critical → danger  (red)
 */
const gaugeVariant = computed<'success' | 'warning' | 'danger'>(() => {
  const level = riskLevel(props.score)
  switch (level) {
    case 'low': return 'success'
    case 'medium': return 'warning'
    case 'high':
    case 'critical': return 'danger'
  }
})

/**
 * Map pixel size to the closest SGauge preset.
 * sm = 48px, md = 72px, lg = 96px.
 * We use the preset closest to the requested size.
 */
const gaugeSize = computed<'sm' | 'md' | 'lg'>(() => {
  if (props.size <= 56) return 'sm'
  if (props.size <= 84) return 'md'
  return 'lg'
})

/**
 * SGauge preset dimensions — used to calculate the CSS scale factor
 * so the rendered gauge matches the requested pixel size.
 */
const presetDimension = computed(() => {
  const dimensions = { sm: 48, md: 72, lg: 96 }
  return dimensions[gaugeSize.value]
})

const scaleFactor = computed(() => props.size / presetDimension.value)
</script>

<template>
  <div
    class="risk-gauge-wrapper"
    :style="{
      width: `${size}px`,
      height: `${size}px`,
    }"
  >
    <div
      class="risk-gauge-inner"
      :style="{
        transform: `scale(${scaleFactor})`,
        transformOrigin: 'top left',
      }"
    >
      <SGauge
        :value="gaugeValue"
        :size="gaugeSize"
        :variant="gaugeVariant"
      />
    </div>
    <!-- Overlay the raw score (0-10) over SGauge's percentage text -->
    <span
      class="risk-gauge-score"
      :style="{ fontSize: `${Math.round(size * 0.36)}px` }"
    >
      {{ score }}
    </span>
  </div>
</template>

<style scoped>
.risk-gauge-wrapper {
  display: inline-flex;
  position: relative;
  flex-shrink: 0;
  align-items: center;
  justify-content: center;
  overflow: hidden;
}

.risk-gauge-inner {
  position: absolute;
  top: 0;
  left: 0;
}

/* Hide SGauge's built-in percentage text so we can show the 0-10 score instead */
.risk-gauge-inner :deep(svg text) {
  display: none;
}

/* Hide SGauge's optional label span */
.risk-gauge-inner :deep(> span) {
  display: none;
}

.risk-gauge-score {
  position: absolute;
  font-family: var(--font-mono);
  font-weight: 700;
  color: var(--color-text-primary);
  pointer-events: none;
  z-index: 1;
}
</style>
