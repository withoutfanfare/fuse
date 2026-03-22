<script setup lang="ts">
import { SSkeleton, SSkeletonText } from '@stuntrocket/ui'

withDefaults(defineProps<{
  /** Layout variant: 'list' for rows, 'cards' for a grid, 'detail' for a page-style layout */
  variant?: 'list' | 'cards' | 'detail'
  /** Number of placeholder items to show */
  count?: number
}>(), {
  variant: 'list',
  count: 4,
})
</script>

<template>
  <div class="content-loader" :class="`content-loader--${variant}`">
    <!-- List variant: table-like rows -->
    <template v-if="variant === 'list'">
      <div v-for="i in count" :key="i" class="loader-row" :style="{ animationDelay: `${i * 80}ms` }">
        <SSkeleton width="32px" height="32px" variant="bar" />
        <div class="loader-row-text">
          <SSkeleton width="60%" height="14px" />
          <SSkeleton width="35%" height="10px" />
        </div>
        <SSkeleton width="48px" height="20px" />
      </div>
    </template>

    <!-- Cards variant: grid of card placeholders -->
    <template v-if="variant === 'cards'">
      <div v-for="i in count" :key="i" class="loader-card" :style="{ animationDelay: `${i * 80}ms` }">
        <SSkeleton width="70%" height="16px" />
        <SSkeleton width="100%" height="40px" />
        <div class="loader-card-footer">
          <SSkeleton width="64px" height="12px" />
          <SSkeleton width="40px" height="12px" />
        </div>
      </div>
    </template>

    <!-- Detail variant: single page layout -->
    <template v-if="variant === 'detail'">
      <div class="loader-detail-header">
        <SSkeleton width="45%" height="20px" />
        <SSkeleton width="25%" height="12px" />
      </div>
      <div class="loader-detail-body">
        <div v-for="i in count" :key="i" class="loader-detail-section" :style="{ animationDelay: `${i * 100}ms` }">
          <SSkeletonText :lines="3" size="sm" width="varied" />
        </div>
      </div>
    </template>
  </div>
</template>

<style scoped>
.content-loader {
  padding: var(--space-2) 0;
}

/* --- List variant --- */
.content-loader--list {
  display: flex;
  flex-direction: column;
  gap: var(--space-3);
}

.loader-row {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  padding: var(--space-3) var(--space-4);
  background: var(--color-surface-panel);
  border: 1px solid var(--color-border-default);
  border-radius: var(--radius-lg);
  opacity: 0;
  animation: loader-fade-in 0.4s ease forwards;
}

.loader-row-text {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: var(--space-1);
}

/* --- Cards variant --- */
.content-loader--cards {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(240px, 1fr));
  gap: var(--space-3);
}

.loader-card {
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
  padding: var(--space-4);
  background: var(--color-surface-panel);
  border: 1px solid var(--color-border-default);
  border-radius: var(--radius-lg);
  opacity: 0;
  animation: loader-fade-in 0.4s ease forwards;
}

.loader-card-footer {
  display: flex;
  gap: var(--space-2);
  margin-top: var(--space-1);
}

/* --- Detail variant --- */
.content-loader--detail {
  display: flex;
  flex-direction: column;
  gap: var(--space-5);
}

.loader-detail-header {
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
}

.loader-detail-body {
  display: flex;
  flex-direction: column;
  gap: var(--space-4);
}

.loader-detail-section {
  padding: var(--space-4);
  background: var(--color-surface-panel);
  border: 1px solid var(--color-border-default);
  border-radius: var(--radius-lg);
  opacity: 0;
  animation: loader-fade-in 0.4s ease forwards;
}

/* Staggered fade-in */
@keyframes loader-fade-in {
  from {
    opacity: 0;
    transform: translateY(4px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}
</style>
