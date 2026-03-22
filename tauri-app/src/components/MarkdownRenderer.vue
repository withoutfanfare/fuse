<script setup lang="ts">
import { computed } from 'vue'
import { marked } from 'marked'
import DOMPurify from 'dompurify'
import { SProse } from '@stuntrocket/ui'

const props = defineProps<{
  content: string
}>()

const renderedHtml = computed(() => {
  const raw = marked.parse(props.content, { async: false }) as string
  return DOMPurify.sanitize(raw)
})
</script>

<template>
  <SProse>
    <div v-html="renderedHtml" />
  </SProse>
</template>
