<script setup lang="ts">
import { computed } from 'vue'
import { SBadge, SStatusDot } from '@stuntrocket/ui'
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

/** Map CI status to SBadge variant. */
const badgeVariant = computed(() => {
  const variantMap: Record<string, 'success' | 'error' | 'warning'> = {
    pass: 'success',
    fail: 'error',
    pending: 'warning',
  }
  return variantMap[overallStatus.value]
})

/** Map CI status to SStatusDot variant. */
const dotVariant = computed(() => {
  const variantMap: Record<string, 'success' | 'error' | 'warning'> = {
    pass: 'success',
    fail: 'error',
    pending: 'warning',
  }
  return variantMap[overallStatus.value]
})
</script>

<template>
  <SBadge :variant="badgeVariant" class="ci-badge">
    <SStatusDot :variant="dotVariant" />
    {{ label }}
  </SBadge>
</template>

<style scoped>
.ci-badge {
  gap: var(--space-1);
}
</style>
