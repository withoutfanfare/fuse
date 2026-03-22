<script setup lang="ts">
import { computed } from 'vue'
import { SSelect } from '@stuntrocket/ui'
import { useGroupsStore } from '../stores/groups'

const groupsStore = useGroupsStore()

const selectedGroupId = defineModel<number | null>('modelValue', { default: null })

/**
 * SSelect works with string modelValue, so we bridge between
 * the numeric group ID and the string value used by SSelect.
 */
const selectValue = computed({
  get() {
    return selectedGroupId.value === null ? '' : String(selectedGroupId.value)
  },
  set(val: string) {
    selectedGroupId.value = val === '' ? null : Number(val)
  },
})
</script>

<template>
  <div v-if="groupsStore.groups.length > 0" class="filter-group">
    <label class="filter-label">Group</label>
    <SSelect v-model="selectValue" size="sm">
      <option value="">All Groups</option>
      <option v-for="group in groupsStore.groups" :key="group.id" :value="String(group.id)">
        {{ group.name }}
      </option>
    </SSelect>
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
</style>
