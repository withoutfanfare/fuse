<script setup lang="ts">
import { computed } from 'vue'
import { SBadge } from '@stuntrocket/ui'
import type { ConflictStatus } from '../types'

const props = defineProps<{
  status: ConflictStatus | null
  loading?: boolean
}>()

/** Map conflict state to SBadge variant. */
const badgeVariant = computed<'error' | 'success' | 'warning'>(() => {
  if (!props.status) return 'warning'
  if (props.status.has_conflicts) return 'error'
  if (props.status.mergeable === 'MERGEABLE') return 'success'
  return 'warning'
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
  <SBadge
    v-if="loading || status"
    :variant="badgeVariant"
    :title="tooltip"
    class="conflict-badge"
  >
    <span class="conflict-icon" :class="{ 'conflict-icon--loading': loading }">{{ icon }}</span>
    {{ label }}
  </SBadge>
</template>

<style scoped>
.conflict-badge {
  cursor: help;
  gap: var(--space-1);
}

.conflict-icon {
  font-size: 11px;
}

.conflict-icon--loading {
  animation: spin 1.5s linear infinite;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}
</style>
