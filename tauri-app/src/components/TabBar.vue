<script setup lang="ts">
import { computed } from 'vue'
import { SSegmentedControl } from '@stuntrocket/ui'

const props = defineProps<{
  tabs: { key: string; label: string }[]
  modelValue: string
}>()

const emit = defineEmits<{
  'update:modelValue': [value: string]
}>()

/** Map TabBar's { key, label } format to SSegmentedControl's { value, label } format */
const options = computed(() =>
  props.tabs.map(tab => ({ label: tab.label, value: tab.key }))
)
</script>

<template>
  <div class="tab-bar">
    <SSegmentedControl
      :options="options"
      :model-value="modelValue"
      @update:model-value="emit('update:modelValue', $event)"
    />
  </div>
</template>

<style scoped>
.tab-bar {
  margin-bottom: var(--space-5);
}
</style>
