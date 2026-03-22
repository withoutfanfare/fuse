<script setup lang="ts">
import { computed } from 'vue'
import { SBadge } from '@stuntrocket/ui'
import { riskLevel } from '../composables/useRiskScore'

const props = defineProps<{
  score: number
}>()

const level = computed(() => riskLevel(props.score))

/** Map risk level to SBadge variant. */
const badgeVariant = computed(() => {
  const variantMap: Record<string, 'success' | 'warning' | 'error'> = {
    low: 'success',
    medium: 'warning',
    high: 'warning',
    critical: 'error',
  }
  return variantMap[level.value] ?? 'default'
})
</script>

<template>
  <SBadge :variant="badgeVariant" class="risk-badge">
    {{ score }}
  </SBadge>
</template>

<style scoped>
.risk-badge {
  min-width: 28px;
  font-weight: 700;
  letter-spacing: 0.02em;
}
</style>
