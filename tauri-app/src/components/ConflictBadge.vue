<script setup lang="ts">
import { computed } from 'vue'
import type { ConflictStatus } from '../types'

const props = defineProps<{
  status: ConflictStatus | null
  loading?: boolean
}>()

const badgeClass = computed(() => {
  if (!props.status) return 'conflict-unknown'
  if (props.status.has_conflicts) return 'conflict-yes'
  if (props.status.mergeable === 'MERGEABLE') return 'conflict-clean'
  return 'conflict-unknown'
})

const label = computed(() => {
  if (props.loading) return 'Checking...'
  if (!props.status) return ''
  if (props.status.has_conflicts) return 'Conflicts'
  if (props.status.mergeable === 'MERGEABLE') return 'Mergeable'
  return 'Unknown'
})

const icon = computed(() => {
  if (props.loading) return '\u25F7' // clock symbol
  if (!props.status) return ''
  if (props.status.has_conflicts) return '\u2716' // cross mark
  if (props.status.mergeable === 'MERGEABLE') return '\u2714' // check mark
  return '\u003F' // question mark
})

const tooltip = computed(() => {
  if (props.loading) return 'Checking merge conflict status...'
  if (!props.status) return ''
  if (props.status.has_conflicts) {
    return `This PR has merge conflicts (state: ${props.status.merge_state_status})`
  }
  if (props.status.mergeable === 'MERGEABLE') {
    return `No conflicts detected (state: ${props.status.merge_state_status})`
  }
  return `Merge status could not be determined (state: ${props.status.merge_state_status})`
})
</script>

<template>
  <span
    v-if="loading || status"
    class="conflict-badge"
    :class="[badgeClass]"
    :title="tooltip"
  >
    <span class="conflict-icon">{{ icon }}</span>
    {{ label }}
  </span>
</template>

<style scoped>
.conflict-badge {
  display: inline-flex;
  align-items: center;
  gap: var(--space-1);
  padding: 2px var(--space-2);
  border-radius: var(--radius-full);
  font-size: 12px;
  font-weight: 600;
  letter-spacing: 0.01em;
  cursor: help;
}

.conflict-icon {
  font-size: 11px;
}

.conflict-clean {
  background: rgba(34, 197, 94, 0.2);
  color: var(--color-status-success);
}

.conflict-yes {
  background: rgba(220, 38, 38, 0.2);
  color: var(--color-status-danger);
}

.conflict-unknown {
  background: rgba(234, 179, 8, 0.2);
  color: var(--color-status-warning);
}

.conflict-unknown .conflict-icon {
  animation: spin 1.5s linear infinite;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}
</style>
