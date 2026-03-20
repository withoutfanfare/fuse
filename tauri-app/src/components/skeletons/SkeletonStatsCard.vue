<template>
  <div class="skeleton-stats-card">
    <div class="skeleton-block skeleton-value"></div>
    <div class="skeleton-block skeleton-label"></div>
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
}

.skeleton-value {
  width: 64px;
  height: 32px;
  margin-bottom: var(--space-2);
  border-radius: var(--radius-sm);
}

.skeleton-label {
  width: 96px;
  height: 14px;
  border-radius: var(--radius-sm);
}
</style>
