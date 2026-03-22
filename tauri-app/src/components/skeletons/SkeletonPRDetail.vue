<script setup lang="ts">
import { SSkeleton } from '@stuntrocket/ui'
</script>

<template>
  <div class="skeleton-pr-detail">
    <!-- Back button placeholder -->
    <SSkeleton width="52px" height="16px" />

    <!-- Header -->
    <div class="skeleton-header">
      <div class="skeleton-header-left">
        <div class="skeleton-title-row">
          <SSkeleton width="40px" height="24px" />
          <SSkeleton width="320px" height="24px" />
        </div>
        <div class="skeleton-meta-row">
          <SSkeleton width="56px" height="14px" />
          <SSkeleton width="100px" height="14px" />
          <SSkeleton width="56px" height="14px" />
          <SSkeleton width="48px" height="20px" />
        </div>
      </div>
      <SSkeleton width="140px" height="34px" />
    </div>

    <!-- Content grid -->
    <div class="skeleton-grid">
      <div class="skeleton-main">
        <!-- Change Summary section -->
        <div class="skeleton-section">
          <SSkeleton width="120px" height="14px" />
          <div class="skeleton-stats">
            <div v-for="n in 3" :key="n" class="skeleton-stat">
              <SSkeleton width="56px" height="28px" />
              <SSkeleton width="72px" height="12px" />
            </div>
          </div>
        </div>

        <!-- Branch section -->
        <div class="skeleton-section">
          <SSkeleton width="120px" height="14px" />
          <div class="skeleton-branch-row">
            <SSkeleton width="140px" height="24px" />
            <span class="skeleton-arrow">&rarr;</span>
            <SSkeleton width="80px" height="24px" />
          </div>
        </div>

        <!-- Timeline section -->
        <div class="skeleton-section">
          <SSkeleton width="120px" height="14px" />
          <div class="skeleton-timeline">
            <div v-for="n in 3" :key="n" class="skeleton-timeline-item">
              <SSkeleton width="64px" height="14px" />
              <SSkeleton width="120px" height="14px" />
            </div>
          </div>
        </div>
      </div>

      <!-- Sidebar -->
      <div class="skeleton-sidebar">
        <div class="skeleton-section">
          <SSkeleton width="120px" height="14px" />
          <SSkeleton width="100%" height="160px" />
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.skeleton-pr-detail {
  max-width: 1200px;
  display: flex;
  flex-direction: column;
  gap: var(--space-4);
}

.skeleton-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: var(--space-2);
}

.skeleton-header-left {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
}

.skeleton-title-row {
  display: flex;
  align-items: center;
  gap: var(--space-3);
}

.skeleton-meta-row {
  display: flex;
  align-items: center;
  gap: var(--space-2);
}

.skeleton-grid {
  display: grid;
  grid-template-columns: 1fr 320px;
  gap: var(--space-6);
}

.skeleton-section {
  margin-bottom: var(--space-6);
  background: var(--color-surface-panel);
  border: 1px solid var(--color-border-default);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-card);
  padding: var(--space-5);
  display: flex;
  flex-direction: column;
  gap: var(--space-3);
}

.skeleton-stats {
  display: flex;
  gap: var(--space-6);
}

.skeleton-stat {
  display: flex;
  flex-direction: column;
  gap: var(--space-1);
}

.skeleton-branch-row {
  display: flex;
  align-items: center;
  gap: var(--space-3);
}

.skeleton-arrow {
  color: var(--color-text-muted);
}

.skeleton-timeline {
  display: flex;
  flex-direction: column;
  gap: var(--space-3);
}

.skeleton-timeline-item {
  display: flex;
  justify-content: space-between;
  padding: var(--space-2) 0;
  border-bottom: 1px solid rgba(255, 255, 255, 0.04);
}

.skeleton-timeline-item:last-child {
  border-bottom: none;
}
</style>
