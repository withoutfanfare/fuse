<script setup lang="ts">
import { computed } from 'vue'

const props = defineProps<{
  additions: number
  deletions: number
}>()

const total = computed(() => props.additions + props.deletions)

const additionPercent = computed(() => {
  if (total.value === 0) return 0
  return (props.additions / total.value) * 100
})

const deletionPercent = computed(() => {
  if (total.value === 0) return 0
  return (props.deletions / total.value) * 100
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
    <div
      class="size-bar-additions"
      :style="{ width: `${additionPercent}%` }"
    />
    <div
      class="size-bar-deletions"
      :style="{ width: `${deletionPercent}%` }"
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

.size-bar-additions {
  background: var(--color-status-success);
  height: 100%;
}

.size-bar-deletions {
  background: var(--color-status-danger);
  height: 100%;
}

.size-bar-empty {
  width: 40px;
  background: var(--color-border-default);
}
</style>
