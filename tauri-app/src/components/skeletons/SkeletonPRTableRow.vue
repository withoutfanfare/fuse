<template>
  <tr v-for="n in 5" :key="n" class="skeleton-row">
    <td class="col-risk">
      <div class="skeleton-block skeleton-badge"></div>
    </td>
    <td class="col-pr">
      <div class="skeleton-pr-cell">
        <div class="skeleton-block skeleton-number"></div>
        <div class="skeleton-block skeleton-title"></div>
      </div>
    </td>
    <td class="col-author">
      <div class="skeleton-block skeleton-author"></div>
    </td>
    <td class="col-branch">
      <div class="skeleton-branch-cell">
        <div class="skeleton-block skeleton-branch-name"></div>
        <span class="skeleton-arrow">&rarr;</span>
        <div class="skeleton-block skeleton-branch-name skeleton-branch-short"></div>
      </div>
    </td>
    <td class="col-size">
      <div class="skeleton-block skeleton-size"></div>
    </td>
    <td class="col-age">
      <div class="skeleton-block skeleton-age"></div>
    </td>
    <td class="col-status">
      <div class="skeleton-block skeleton-status"></div>
    </td>
    <td class="col-review">
      <div class="skeleton-block skeleton-review"></div>
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

.skeleton-badge {
  width: 40px;
  height: 22px;
  border-radius: var(--radius-full);
}

.skeleton-pr-cell {
  display: flex;
  align-items: center;
  gap: var(--space-2);
}

.skeleton-number {
  width: 36px;
  height: 14px;
  border-radius: var(--radius-sm);
}

.skeleton-title {
  width: 180px;
  height: 14px;
  border-radius: var(--radius-sm);
}

.skeleton-author {
  width: 80px;
  height: 14px;
  border-radius: var(--radius-sm);
}

.skeleton-branch-cell {
  display: flex;
  align-items: center;
  gap: var(--space-1);
}

.skeleton-branch-name {
  width: 100px;
  height: 20px;
  border-radius: var(--radius-sm);
}

.skeleton-branch-short {
  width: 60px;
}

.skeleton-arrow {
  color: var(--color-text-muted);
  font-size: 11px;
}

.skeleton-size {
  width: 90px;
  height: 14px;
  border-radius: var(--radius-sm);
}

.skeleton-age {
  width: 32px;
  height: 14px;
  border-radius: var(--radius-sm);
}

.skeleton-status {
  width: 52px;
  height: 20px;
  border-radius: var(--radius-full);
}

.skeleton-review {
  width: 64px;
  height: 20px;
  border-radius: var(--radius-full);
}
</style>
