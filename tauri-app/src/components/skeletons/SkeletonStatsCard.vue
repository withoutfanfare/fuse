<script setup lang="ts">
import { SSkeleton } from '@stuntrocket/ui'
</script>

<template>
  <div class="skeleton-stats-card">
    <SSkeleton width="64px" height="32px" />
    <SSkeleton width="96px" height="14px" />
  </div>
</template>

<style scoped>
.skeleton-stats-card {
  background: var(--color-surface-panel);
  border: 1px solid var(--color-border-default);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-card);
  padding: var(--space-3) var(--space-4);
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
}
</style>
