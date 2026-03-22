<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { FileText } from 'lucide-vue-next'
import { SDropdownMenu } from '@stuntrocket/ui'
import type { SDropdownMenuItem } from '@stuntrocket/ui'
import type { ReviewTemplate } from '../types'

const emit = defineEmits<{
  'select-template': [body: string]
}>()

const templates = ref<ReviewTemplate[]>([])

onMounted(async () => {
  try {
    templates.value = await invoke<ReviewTemplate[]>('list_templates')
  } catch {
    templates.value = []
  }
})

const menuItems = computed<SDropdownMenuItem[]>(() => {
  if (templates.value.length === 0) {
    return [{ label: 'No templates configured', value: '__empty__', disabled: true }]
  }
  return templates.value.map(tpl => ({
    label: tpl.name,
    value: String(tpl.id),
  }))
})

function onSelect(value: string) {
  if (value === '__empty__') return
  const tpl = templates.value.find(t => String(t.id) === value)
  if (tpl) {
    emit('select-template', tpl.body)
  }
}
</script>

<template>
  <SDropdownMenu
    :items="menuItems"
    align="right"
    @select="onSelect"
  >
    <template #trigger="{ toggle }">
      <button class="btn-template-trigger" title="Quick review templates" @click="toggle">
        <FileText :size="14" />
        Template
      </button>
    </template>
  </SDropdownMenu>
</template>

<style scoped>
.btn-template-trigger {
  display: inline-flex;
  align-items: center;
  gap: var(--space-1);
  background: var(--color-surface-raised);
  color: var(--color-text-secondary);
  font-size: 12px;
  font-weight: 500;
  padding: var(--space-1) var(--space-2);
  border-radius: var(--radius-md);
  border: 1px solid var(--color-border-default);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.btn-template-trigger:hover {
  background: var(--color-surface-hover);
  color: var(--color-text-primary);
  border-color: var(--color-border-hover);
}

.btn-template-trigger:focus-visible {
  box-shadow: 0 0 0 2px var(--color-accent-muted);
  outline: none;
}
</style>
