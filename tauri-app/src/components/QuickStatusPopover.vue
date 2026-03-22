<script setup lang="ts">
import { computed } from 'vue'
import { SDropdownMenu } from '@stuntrocket/ui'
import type { SDropdownMenuItem } from '@stuntrocket/ui'
import type { ReviewStatus } from '../types'

const props = defineProps<{
  currentStatus: ReviewStatus | null
  anchorRect: DOMRect
}>()

const emit = defineEmits<{
  'select': [status: ReviewStatus]
  'close': []
}>()

/** All available review statuses with display labels */
const statusOptions: { value: ReviewStatus; label: string; colourClass: string }[] = [
  { value: 'pending', label: 'Pending', colourClass: 'opt-pending' },
  { value: 'in_progress', label: 'In Progress', colourClass: 'opt-in_progress' },
  { value: 'reviewed', label: 'Reviewed', colourClass: 'opt-reviewed' },
  { value: 'approved', label: 'Approved', colourClass: 'opt-approved' },
  { value: 'changes_requested', label: 'Changes Requested', colourClass: 'opt-changes_requested' },
]

const menuItems = computed<SDropdownMenuItem[]>(() =>
  statusOptions.map(opt => ({
    label: `${opt.label}${props.currentStatus === opt.value ? ' \u2713' : ''}`,
    value: opt.value,
  })),
)

function onSelect(value: string) {
  emit('select', value as ReviewStatus)
  emit('close')
}
</script>

<template>
  <SDropdownMenu
    :items="menuItems"
    align="left"
    @select="onSelect"
  >
    <template #trigger="{ toggle }">
      <button
        class="status-trigger"
        role="listbox"
        aria-label="Change review status"
        @click="toggle"
      >
        Set Status
      </button>
    </template>
  </SDropdownMenu>
</template>

<style scoped>
.status-trigger {
  font-size: 11px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  color: var(--color-text-muted);
  background: none;
  border: none;
  cursor: pointer;
  padding: var(--space-1) var(--space-2);
  border-radius: var(--radius-md);
  transition: all var(--transition-fast);
}

.status-trigger:hover {
  background: var(--color-surface-hover);
  color: var(--color-text-primary);
}
</style>
