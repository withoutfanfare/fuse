<script setup lang="ts">
import { SSkeleton } from '@stuntrocket/ui'
</script>

<template>
  <tr v-for="n in 5" :key="n" class="skeleton-row">
    <td class="col-risk">
      <SSkeleton width="40px" height="22px" />
    </td>
    <td class="col-pr">
      <div class="skeleton-pr-cell">
        <SSkeleton width="36px" height="14px" />
        <SSkeleton width="180px" height="14px" />
      </div>
    </td>
    <td class="col-author">
      <SSkeleton width="80px" height="14px" />
    </td>
    <td class="col-branch">
      <div class="skeleton-branch-cell">
        <SSkeleton width="100px" height="20px" />
        <span class="skeleton-arrow">&rarr;</span>
        <SSkeleton width="60px" height="20px" />
      </div>
    </td>
    <td class="col-size">
      <SSkeleton width="90px" height="14px" />
    </td>
    <td class="col-age">
      <SSkeleton width="32px" height="14px" />
    </td>
    <td class="col-status">
      <SSkeleton width="52px" height="20px" />
    </td>
    <td class="col-review">
      <SSkeleton width="64px" height="20px" />
    </td>
  </tr>
</template>

<style scoped>
.skeleton-row td {
  padding: var(--space-3);
  border-bottom: 1px solid rgba(255, 255, 255, 0.04);
  vertical-align: middle;
}

.skeleton-row:last-child td {
  border-bottom: none;
}

.skeleton-pr-cell {
  display: flex;
  align-items: center;
  gap: var(--space-2);
}

.skeleton-branch-cell {
  display: flex;
  align-items: center;
  gap: var(--space-1);
}

.skeleton-arrow {
  color: var(--color-text-muted);
  font-size: 11px;
}
</style>
