<script setup lang="ts">
import { computed } from 'vue'
import { riskColour } from '../composables/useRiskScore'

const props = withDefaults(defineProps<{
  score: number
  size?: number
}>(), {
  size: 36,
})

const radius = computed(() => (props.size - 4) / 2)
const centre = computed(() => props.size / 2)

/* 270-degree arc = 75 % of the full circumference */
const fullCircumference = computed(() => 2 * Math.PI * radius.value)
const arcLength = computed(() => fullCircumference.value * 0.75)

/* stroke-dasharray: visible arc + invisible gap */
const trackDasharray = computed(() => `${arcLength.value} ${fullCircumference.value}`)

/* Value arc fills proportionally to score / 10 */
const valueDash = computed(() => (props.score / 10) * arcLength.value)
const valueDasharray = computed(() => `${valueDash.value} ${fullCircumference.value}`)

/* Rotate so the gap sits at the bottom centre (start at 135 deg) */
const rotation = computed(() => `rotate(135 ${centre.value} ${centre.value})`)

const colour = computed(() => riskColour(props.score))

/* Font size scales with the gauge */
const fontSize = computed(() => Math.round(props.size * 0.36))
</script>

<template>
  <svg
    class="risk-gauge"
    :width="size"
    :height="size"
    :viewBox="`0 0 ${size} ${size}`"
  >
    <!-- Track circle -->
    <circle
      class="gauge-track"
      :cx="centre"
      :cy="centre"
      :r="radius"
      fill="none"
      :stroke-width="2"
      stroke-linecap="round"
      :stroke-dasharray="trackDasharray"
      :transform="rotation"
    />
    <!-- Value arc -->
    <circle
      class="gauge-value"
      :cx="centre"
      :cy="centre"
      :r="radius"
      fill="none"
      :stroke-width="2"
      stroke-linecap="round"
      :stroke-dasharray="valueDasharray"
      :transform="rotation"
      :style="{ stroke: colour }"
    />
    <!-- Centred score text -->
    <text
      class="gauge-text"
      :x="centre"
      :y="centre"
      text-anchor="middle"
      dominant-baseline="central"
      :style="{ fill: colour, fontSize: `${fontSize}px` }"
    >
      {{ score }}
    </text>
  </svg>
</template>

<style scoped>
.risk-gauge {
  display: inline-block;
  flex-shrink: 0;
}

.gauge-track {
  stroke: var(--color-risk-gauge-track);
}

.gauge-value {
  transition: stroke-dashoffset 0.6s ease, stroke 0.4s ease;
}

.gauge-text {
  font-family: var(--font-mono);
  font-weight: 700;
}
</style>
