<script setup lang="ts">
import { computed } from 'vue'
import type { CiCheck } from '../types'

const props = defineProps<{
  checks: CiCheck[]
}>()

const passed = computed(() =>
  props.checks.filter(c => c.conclusion === 'SUCCESS' || c.conclusion === 'success').length
)

const failed = computed(() =>
  props.checks.filter(c => c.conclusion === 'FAILURE' || c.conclusion === 'failure').length
)

const pending = computed(() =>
  props.checks.filter(c => !c.conclusion || c.state === 'PENDING' || c.state === 'pending' || c.state === 'IN_PROGRESS' || c.state === 'in_progress').length
)

const overallStatus = computed<'pass' | 'fail' | 'pending'>(() => {
  if (failed.value > 0) return 'fail'
  if (pending.value > 0) return 'pending'
  return 'pass'
})

const label = computed(() => {
  if (overallStatus.value === 'fail') return 'Failed'
  if (overallStatus.value === 'pending') return 'Pending'
  return `${passed.value}/${props.checks.length} passed`
})
</script>

<template>
  <span class="ci-badge" :class="[`ci-${overallStatus}`]">
    <span class="ci-icon">
      <template v-if="overallStatus === 'pass'">&#10003;</template>
      <template v-else-if="overallStatus === 'fail'">&#10007;</template>
      <template v-else>&#9719;</template>
    </span>
    {{ label }}
  </span>
</template>

<style scoped>
.ci-badge {
  display: inline-flex;
  align-items: center;
  gap: var(--space-1);
  padding: 2px var(--space-2);
  border-radius: var(--radius-full);
  font-size: 12px;
  font-weight: 600;
  letter-spacing: 0.01em;
}

.ci-icon {
  font-size: 11px;
}

.ci-pass {
  background: rgba(34, 197, 94, 0.2);
  color: var(--color-status-success);
}

.ci-fail {
  background: rgba(220, 38, 38, 0.2);
  color: var(--color-status-danger);
}

.ci-pending {
  background: rgba(234, 179, 8, 0.2);
  color: var(--color-status-warning);
}

.ci-pending .ci-icon {
  animation: spin 1.5s linear infinite;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}
</style>
