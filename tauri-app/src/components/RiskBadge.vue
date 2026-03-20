<script setup lang="ts">
import { computed } from 'vue'
import { riskLevel } from '../composables/useRiskScore'

const props = defineProps<{
  score: number
}>()

const level = computed(() => riskLevel(props.score))
</script>

<template>
  <span class="risk-badge" :class="[`risk-${level}`]">
    {{ score }}
  </span>
</template>

<style scoped>
.risk-badge {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 28px;
  height: 24px;
  padding: 0 var(--space-2);
  border-radius: var(--radius-full);
  font-size: 12px;
  font-weight: 700;
  letter-spacing: 0.02em;
}

.risk-low {
  background: rgba(34, 197, 94, 0.2);
  color: var(--color-risk-low);
}
.risk-medium {
  background: rgba(234, 179, 8, 0.2);
  color: var(--color-risk-medium);
}
.risk-high {
  background: rgba(234, 179, 8, 0.2);
  color: var(--color-risk-high);
}
.risk-critical {
  background: rgba(220, 38, 38, 0.2);
  color: var(--color-risk-critical);
}
</style>
