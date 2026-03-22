<script setup lang="ts">
import { computed, ref, onMounted, onUnmounted } from 'vue'
import { SCard } from '@stuntrocket/ui'
import type { VelocityPoint } from '../types'

const props = defineProps<{
  data: VelocityPoint[]
}>()

const containerRef = ref<HTMLElement | null>(null)
const containerWidth = ref(600)

const padding = { top: 20, right: 20, bottom: 40, left: 40 }
const chartHeight = 200

const chartWidth = computed(() => Math.max(containerWidth.value, 300))
const innerWidth = computed(() => chartWidth.value - padding.left - padding.right)
const innerHeight = chartHeight - padding.top - padding.bottom

const maxY = computed(() => {
  if (props.data.length === 0) return 1
  const allVals = props.data.flatMap(d => [d.reviewed, d.merged])
  return Math.max(...allVals, 1)
})

function xScale(i: number): number {
  if (props.data.length <= 1) return padding.left
  return padding.left + (i / (props.data.length - 1)) * innerWidth.value
}

function yScale(val: number): number {
  return padding.top + innerHeight - (val / maxY.value) * innerHeight
}

const reviewedLine = computed(() => {
  if (props.data.length === 0) return ''
  return props.data.map((d, i) => `${i === 0 ? 'M' : 'L'}${xScale(i).toFixed(1)},${yScale(d.reviewed).toFixed(1)}`).join(' ')
})

const mergedLine = computed(() => {
  if (props.data.length === 0) return ''
  return props.data.map((d, i) => `${i === 0 ? 'M' : 'L'}${xScale(i).toFixed(1)},${yScale(d.merged).toFixed(1)}`).join(' ')
})

const reviewedArea = computed(() => {
  if (props.data.length === 0) return ''
  const baseline = yScale(0)
  const line = props.data.map((d, i) => `L${xScale(i).toFixed(1)},${yScale(d.reviewed).toFixed(1)}`).join(' ')
  return `M${xScale(0).toFixed(1)},${baseline} ${line} L${xScale(props.data.length - 1).toFixed(1)},${baseline} Z`
})

const mergedArea = computed(() => {
  if (props.data.length === 0) return ''
  const baseline = yScale(0)
  const line = props.data.map((d, i) => `L${xScale(i).toFixed(1)},${yScale(d.merged).toFixed(1)}`).join(' ')
  return `M${xScale(0).toFixed(1)},${baseline} ${line} L${xScale(props.data.length - 1).toFixed(1)},${baseline} Z`
})

const yTicks = computed(() => {
  const ticks: number[] = []
  const step = Math.max(1, Math.ceil(maxY.value / 4))
  for (let v = 0; v <= maxY.value; v += step) {
    ticks.push(v)
  }
  return ticks
})

const xLabels = computed(() => {
  if (props.data.length === 0) return []
  const step = Math.max(1, Math.floor(props.data.length / 6))
  const labels: { x: number; label: string }[] = []
  for (let i = 0; i < props.data.length; i += step) {
    const d = new Date(props.data[i].date)
    labels.push({
      x: xScale(i),
      label: d.toLocaleDateString('en-GB', { day: 'numeric', month: 'short' }),
    })
  }
  return labels
})

let resizeObserver: ResizeObserver | null = null

onMounted(() => {
  if (containerRef.value) {
    containerWidth.value = containerRef.value.clientWidth
    resizeObserver = new ResizeObserver(entries => {
      for (const entry of entries) {
        containerWidth.value = entry.contentRect.width
      }
    })
    resizeObserver.observe(containerRef.value)
  }
})

onUnmounted(() => {
  resizeObserver?.disconnect()
})
</script>

<template>
  <SCard variant="content">
    <div ref="containerRef" class="velocity-chart-container">
      <svg
        :width="chartWidth"
        :height="chartHeight"
        :viewBox="`0 0 ${chartWidth} ${chartHeight}`"
        class="velocity-chart"
      >
        <defs>
          <linearGradient id="vel-reviewed-grad" x1="0" y1="0" x2="0" y2="1">
            <stop offset="0%" stop-color="var(--color-accent)" stop-opacity="0.25" />
            <stop offset="100%" stop-color="var(--color-accent)" stop-opacity="0" />
          </linearGradient>
          <linearGradient id="vel-merged-grad" x1="0" y1="0" x2="0" y2="1">
            <stop offset="0%" stop-color="var(--color-status-success)" stop-opacity="0.2" />
            <stop offset="100%" stop-color="var(--color-status-success)" stop-opacity="0" />
          </linearGradient>
        </defs>

        <!-- Grid lines -->
        <line
          v-for="tick in yTicks"
          :key="`grid-${tick}`"
          :x1="padding.left"
          :x2="chartWidth - padding.right"
          :y1="yScale(tick)"
          :y2="yScale(tick)"
          stroke="rgba(255,255,255,0.06)"
          stroke-width="1"
        />

        <!-- Y-axis labels -->
        <text
          v-for="tick in yTicks"
          :key="`ylabel-${tick}`"
          :x="padding.left - 8"
          :y="yScale(tick) + 4"
          text-anchor="end"
          class="axis-label"
        >
          {{ tick }}
        </text>

        <!-- X-axis labels -->
        <text
          v-for="lbl in xLabels"
          :key="lbl.label"
          :x="lbl.x"
          :y="chartHeight - 8"
          text-anchor="middle"
          class="axis-label"
        >
          {{ lbl.label }}
        </text>

        <!-- Area fills -->
        <path v-if="reviewedArea" :d="reviewedArea" fill="url(#vel-reviewed-grad)" />
        <path v-if="mergedArea" :d="mergedArea" fill="url(#vel-merged-grad)" />

        <!-- Lines -->
        <path
          v-if="reviewedLine"
          :d="reviewedLine"
          fill="none"
          stroke="var(--color-accent)"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
        />
        <path
          v-if="mergedLine"
          :d="mergedLine"
          fill="none"
          stroke="var(--color-status-success)"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
        />
      </svg>

      <div class="velocity-legend">
        <span class="legend-item">
          <span class="legend-swatch" style="background: var(--color-accent)" />
          Reviewed
        </span>
        <span class="legend-item">
          <span class="legend-swatch" style="background: var(--color-status-success)" />
          Merged
        </span>
      </div>
    </div>
  </SCard>
</template>

<style scoped>
.velocity-chart-container {
  width: 100%;
}

.velocity-chart {
  display: block;
  width: 100%;
}

.axis-label {
  fill: var(--color-text-muted);
  font-size: 11px;
  font-family: var(--font-mono);
}

.velocity-legend {
  display: flex;
  gap: var(--space-4);
  justify-content: center;
  margin-top: var(--space-2);
}

.legend-item {
  display: flex;
  align-items: center;
  gap: var(--space-1);
  font-size: 12px;
  color: var(--color-text-secondary);
}

.legend-swatch {
  width: 12px;
  height: 3px;
  border-radius: 2px;
  display: inline-block;
}
</style>
