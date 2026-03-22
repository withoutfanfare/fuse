<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { FileText } from 'lucide-vue-next'
import { SDropdownMenu, SButton } from '@stuntrocket/ui'
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
      <SButton variant="secondary" size="sm" title="Quick review templates" @click="toggle">
        <FileText :size="14" />
        Template
      </SButton>
    </template>
  </SDropdownMenu>
</template>

