<script setup lang="ts">
import { SKeyboardShortcuts } from '@stuntrocket/ui'
import type { ShortcutGroup } from '@stuntrocket/ui'

defineProps<{
  visible: boolean
}>()

const emit = defineEmits<{
  close: []
}>()

const groups: ShortcutGroup[] = [
  {
    title: 'Navigation',
    shortcuts: [
      { keys: '1', description: 'Go to Dashboard' },
      { keys: '2', description: 'Go to Pull Requests' },
      { keys: '3', description: 'Go to Repositories' },
      { keys: '4', description: 'Go to Settings' },
      { keys: '5', description: 'Go to Aggregate view' },
    ],
  },
  {
    title: 'Actions',
    shortcuts: [
      { keys: '/', description: 'Focus search' },
      { keys: 'r', description: 'Sync all repositories' },
      { keys: '\u2318K', description: 'Open command palette' },
      { keys: '?', description: 'Toggle this overlay' },
      { keys: 'Esc', description: 'Close overlay / blur input' },
    ],
  },
  {
    title: 'Review Workflow',
    shortcuts: [
      { keys: '] / n', description: 'Next file in diff' },
      { keys: '[ / p', description: 'Previous file in diff' },
      { keys: 'c', description: 'Toggle checklist item' },
      { keys: 'a', description: 'Jump to next annotation' },
      { keys: '\u21E7\u23CE', description: 'Complete review & advance' },
    ],
  },
]
</script>

<template>
  <SKeyboardShortcuts
    :open="visible"
    :groups="groups"
    @close="emit('close')"
  />
</template>
