<template>
  <div class="skeleton-pr-detail">
    <!-- Back button placeholder -->
    <div class="skeleton-block skeleton-back"></div>

    <!-- Header -->
    <div class="skeleton-header">
      <div class="skeleton-header-left">
        <div class="skeleton-title-row">
          <div class="skeleton-block skeleton-risk-badge"></div>
          <div class="skeleton-block skeleton-title"></div>
        </div>
        <div class="skeleton-meta-row">
          <div class="skeleton-block skeleton-meta-item"></div>
          <div class="skeleton-block skeleton-meta-item skeleton-meta-wide"></div>
          <div class="skeleton-block skeleton-meta-item"></div>
          <div class="skeleton-block skeleton-meta-state"></div>
        </div>
      </div>
      <div class="skeleton-block skeleton-github-btn"></div>
    </div>

    <!-- Content grid -->
    <div class="skeleton-grid">
      <div class="skeleton-main">
        <!-- Change Summary section -->
        <div class="skeleton-section">
          <div class="skeleton-block skeleton-section-title"></div>
          <div class="skeleton-stats">
            <div v-for="n in 3" :key="n" class="skeleton-stat">
              <div class="skeleton-block skeleton-stat-value"></div>
              <div class="skeleton-block skeleton-stat-label"></div>
            </div>
          </div>
        </div>

        <!-- Branch section -->
        <div class="skeleton-section">
          <div class="skeleton-block skeleton-section-title"></div>
          <div class="skeleton-branch-row">
            <div class="skeleton-block skeleton-branch-code"></div>
            <span class="skeleton-arrow">&rarr;</span>
            <div class="skeleton-block skeleton-branch-code skeleton-branch-short"></div>
          </div>
        </div>

        <!-- Timeline section -->
        <div class="skeleton-section">
          <div class="skeleton-block skeleton-section-title"></div>
          <div class="skeleton-timeline">
            <div v-for="n in 3" :key="n" class="skeleton-timeline-item">
              <div class="skeleton-block skeleton-timeline-label"></div>
              <div class="skeleton-block skeleton-timeline-value"></div>
            </div>
          </div>
        </div>
      </div>

      <!-- Sidebar -->
      <div class="skeleton-sidebar">
        <div class="skeleton-section">
          <div class="skeleton-block skeleton-section-title"></div>
          <div class="skeleton-block skeleton-sidebar-content"></div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.skeleton-pr-detail {
  max-width: 1200px;
}

.skeleton-back {
  width: 52px;
  height: 16px;
  margin-bottom: var(--space-4);
  border-radius: var(--radius-sm);
}

.skeleton-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: var(--space-6);
}

.skeleton-header-left {
  flex: 1;
}

.skeleton-title-row {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  margin-bottom: var(--space-2);
}

.skeleton-risk-badge {
  width: 40px;
  height: 24px;
  border-radius: var(--radius-full);
}

.skeleton-title {
  width: 320px;
  height: 24px;
  border-radius: var(--radius-sm);
}

.skeleton-meta-row {
  display: flex;
  align-items: center;
  gap: var(--space-2);
}

.skeleton-meta-item {
  width: 56px;
  height: 14px;
  border-radius: var(--radius-sm);
}

.skeleton-meta-wide {
  width: 100px;
}

.skeleton-meta-state {
  width: 48px;
  height: 20px;
  border-radius: var(--radius-full);
}

.skeleton-github-btn {
  width: 140px;
  height: 34px;
  border-radius: var(--radius-md);
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
}

.skeleton-section-title {
  width: 120px;
  height: 14px;
  margin-bottom: var(--space-3);
  border-radius: var(--radius-sm);
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

.skeleton-stat-value {
  width: 56px;
  height: 28px;
  border-radius: var(--radius-sm);
}

.skeleton-stat-label {
  width: 72px;
  height: 12px;
  border-radius: var(--radius-sm);
}

.skeleton-branch-row {
  display: flex;
  align-items: center;
  gap: var(--space-3);
}

.skeleton-branch-code {
  width: 140px;
  height: 24px;
  border-radius: var(--radius-sm);
}

.skeleton-branch-short {
  width: 80px;
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

.skeleton-timeline-label {
  width: 64px;
  height: 14px;
  border-radius: var(--radius-sm);
}

.skeleton-timeline-value {
  width: 120px;
  height: 14px;
  border-radius: var(--radius-sm);
}

.skeleton-sidebar-content {
  width: 100%;
  height: 160px;
  border-radius: var(--radius-md);
}
</style>
