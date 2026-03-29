<script setup lang="ts">
import { computed } from 'vue'
import { SBadge } from '@stuntrocket/ui'
import type { ConflictRiskEntry } from '../types'

const props = defineProps<{
  risks: ConflictRiskEntry[]
}>()

const emit = defineEmits<{
  click: []
}>()

const totalOverlap = computed(() => {
  const files = new Set<string>()
  for (const r of props.risks) {
    for (const f of r.overlapping_files) {
      files.add(f)
    }
  }
  return files.size
})

const otherPrNumbers = computed(() =>
  [...new Set(props.risks.map((r) => r.other_pr_number))],
)

const tooltip = computed(() => {
  if (props.risks.length === 0) return ''
  const prs = otherPrNumbers.value.map((n) => `#${n}`).join(', ')
  return `${totalOverlap.value} file(s) overlap with ${prs}`
})
</script>

<template>
  <SBadge
    v-if="risks.length > 0"
    variant="warning"
    :title="tooltip"
    class="conflict-risk-badge"
    @click.stop="emit('click')"
  >
    <span class="conflict-risk-icon">&#x26A0;</span>
    {{ totalOverlap }} file overlap
  </SBadge>
</template>

<style scoped>
.conflict-risk-badge {
  cursor: pointer;
  gap: var(--space-1);
}

.conflict-risk-icon {
  font-size: 11px;
}
</style>
