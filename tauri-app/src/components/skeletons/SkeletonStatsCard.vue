<script setup lang="ts">
import { SSkeleton } from '@stuntrocket/ui'
</script>

<template>
  <div class="skeleton-stats-card">
    <SSkeleton width="48px" height="22px" />
    <SSkeleton width="72px" height="12px" />
  </div>
</template>

<style scoped>
.skeleton-stats-card {
  background: var(--color-surface-panel);
  border: 1px solid var(--color-border-default);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-card);
  padding: var(--space-2) var(--space-3);
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: var(--space-1);
}
</style>
