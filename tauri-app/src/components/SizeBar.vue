<script setup lang="ts">
import { computed } from 'vue'
import { SProgressBar } from '@stuntrocket/ui'

const props = defineProps<{
  additions: number
  deletions: number
}>()

const total = computed(() => props.additions + props.deletions)

/** Normalised addition ratio (0-1) for SProgressBar. */
const additionRatio = computed(() => {
  if (total.value === 0) return 0
  return props.additions / total.value
})

/** Normalised deletion ratio (0-1) for SProgressBar. */
const deletionRatio = computed(() => {
  if (total.value === 0) return 0
  return props.deletions / total.value
})

/**
 * Bar width scales proportionally with total changes.
 * Caps at 100px (max-width), floors at 40px (min-width).
 * Uses a logarithmic scale so massive PRs don't dominate.
 */
const barWidth = computed(() => {
  if (total.value === 0) return 40
  // log scale: 1 change = 40px, ~1000+ changes = 100px
  const scaled = 40 + (60 * Math.min(Math.log10(total.value + 1) / 3, 1))
  return Math.round(scaled)
})
</script>

<template>
  <div
    v-if="total > 0"
    class="size-bar"
    :style="{ width: `${barWidth}px` }"
    :title="`+${additions} / -${deletions}`"
  >
    <SProgressBar
      :value="1"
      variant="success"
      size="sm"
      class="size-bar-additions"
      :style="{ width: `${additionRatio * 100}%` }"
    />
    <SProgressBar
      :value="1"
      variant="danger"
      size="sm"
      class="size-bar-deletions"
      :style="{ width: `${deletionRatio * 100}%` }"
    />
  </div>
  <div v-else class="size-bar size-bar-empty" />
</template>

<style scoped>
.size-bar {
  display: flex;
  height: 4px;
  border-radius: 2px;
  overflow: hidden;
  min-width: 40px;
  max-width: 100px;
  margin-top: var(--space-1);
}

.size-bar-additions,
.size-bar-deletions {
  height: 100%;
  flex-shrink: 0;
}

.size-bar-empty {
  width: 40px;
  background: var(--color-border-default);
}
</style>
