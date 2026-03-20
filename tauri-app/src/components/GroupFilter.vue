<script setup lang="ts">
import { useGroupsStore } from '../stores/groups'

const groupsStore = useGroupsStore()

const selectedGroupId = defineModel<number | null>('modelValue', { default: null })
</script>

<template>
  <div v-if="groupsStore.groups.length > 0" class="filter-group">
    <label class="filter-label">Group</label>
    <select v-model="selectedGroupId" class="filter-select">
      <option :value="null">All Groups</option>
      <option v-for="group in groupsStore.groups" :key="group.id" :value="group.id">
        {{ group.name }}
      </option>
    </select>
  </div>
</template>

<style scoped>
.filter-group {
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
}

.filter-label {
  font-size: 12px;
  font-weight: 600;
  color: var(--color-text-muted);
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.filter-select {
  min-width: 160px;
  background: var(--color-surface-input);
  border: 1px solid var(--color-border-default);
  border-radius: var(--radius-md);
  padding: var(--space-2) var(--space-3);
  color: var(--color-text-primary);
  font-size: 13px;
  transition: border-color var(--transition-fast);
}

.filter-select:focus {
  border-color: var(--color-border-focus);
  box-shadow: 0 0 0 2px var(--color-accent-muted);
  outline: none;
}
</style>
