<script setup lang="ts">
import { computed } from 'vue'
import { SCard } from '@stuntrocket/ui'
import type { AgeBucket } from '../types'

const props = defineProps<{
  buckets: AgeBucket[]
}>()

const maxCount = computed(() => Math.max(...props.buckets.map(b => b.count), 1))

const bucketColours: Record<string, string> = {
  '< 1 day': 'var(--color-status-success)',
  '1-3 days': 'var(--color-status-info)',
  '3-7 days': 'var(--color-status-warning)',
  '7-14 days': '#f97316',
  '14+ days': 'var(--color-status-danger)',
}

function getColour(label: string): string {
  return bucketColours[label] ?? 'var(--color-text-muted)'
}
</script>

<template>
  <SCard variant="content">
    <div class="age-heatmap">
      <div
        v-for="bucket in buckets"
        :key="bucket.label"
        class="heatmap-row"
      >
        <span class="heatmap-label">{{ bucket.label }}</span>
        <div class="heatmap-bar-track">
          <div
            class="heatmap-bar-fill"
            :style="{
              width: `${(bucket.count / maxCount) * 100}%`,
              backgroundColor: getColour(bucket.label),
            }"
          />
        </div>
        <span class="heatmap-count" :style="{ color: getColour(bucket.label) }">{{ bucket.count }}</span>
      </div>
    </div>
  </SCard>
</template>

<style scoped>
.age-heatmap {
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
}

.heatmap-row {
  display: flex;
  align-items: center;
  gap: var(--space-3);
}

.heatmap-label {
  width: 80px;
  flex-shrink: 0;
  font-size: 13px;
  color: var(--color-text-secondary);
  text-align: right;
  font-weight: 500;
}

.heatmap-bar-track {
  flex: 1;
  height: 20px;
  background: rgba(255, 255, 255, 0.04);
  border-radius: var(--radius-sm);
  overflow: hidden;
}

.heatmap-bar-fill {
  height: 100%;
  border-radius: var(--radius-sm);
  min-width: 2px;
  transition: width var(--transition-normal);
  opacity: 0.85;
}

.heatmap-count {
  width: 36px;
  flex-shrink: 0;
  font-size: 13px;
  font-weight: 700;
  font-family: var(--font-mono);
  text-align: right;
}
</style>
