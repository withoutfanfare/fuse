<script setup lang="ts">
import { computed } from 'vue'
import { marked } from 'marked'
import DOMPurify from 'dompurify'

const props = defineProps<{
  content: string
}>()

const renderedHtml = computed(() => {
  const raw = marked.parse(props.content, { async: false }) as string
  return DOMPurify.sanitize(raw)
})
</script>

<template>
  <div class="markdown-body" v-html="renderedHtml" />
</template>

<style scoped>
.markdown-body {
  font-size: 14px;
  line-height: 1.6;
  color: var(--color-text-secondary);
  word-wrap: break-word;
}

.markdown-body :deep(h1) {
  font-size: 20px;
  font-weight: 700;
  color: var(--color-text-primary);
  margin: var(--space-4) 0 var(--space-2);
  padding-bottom: var(--space-2);
  border-bottom: 1px solid var(--color-border-default);
}

.markdown-body :deep(h2) {
  font-size: 17px;
  font-weight: 600;
  color: var(--color-text-primary);
  margin: var(--space-4) 0 var(--space-2);
  padding-bottom: var(--space-1);
  border-bottom: 1px solid rgba(255, 255, 255, 0.04);
}

.markdown-body :deep(h3) {
  font-size: 15px;
  font-weight: 600;
  color: var(--color-text-primary);
  margin: var(--space-3) 0 var(--space-2);
}

.markdown-body :deep(p) {
  margin: 0 0 var(--space-3);
}

.markdown-body :deep(a) {
  color: var(--color-accent);
  text-decoration: none;
}

.markdown-body :deep(a:hover) {
  text-decoration: underline;
}

.markdown-body :deep(code) {
  background: var(--color-surface-raised);
  padding: 1px var(--space-1);
  border-radius: var(--radius-sm);
  font-family: var(--font-mono);
  font-size: 12px;
  color: var(--color-text-secondary);
}

.markdown-body :deep(pre) {
  background: var(--color-surface-raised);
  padding: var(--space-3);
  border-radius: var(--radius-md);
  overflow-x: auto;
  margin: 0 0 var(--space-3);
  border: 1px solid var(--color-border-default);
}

.markdown-body :deep(pre code) {
  background: none;
  padding: 0;
  border-radius: 0;
  font-size: 12px;
  line-height: 1.5;
}

.markdown-body :deep(ul),
.markdown-body :deep(ol) {
  padding-left: var(--space-5);
  margin: 0 0 var(--space-3);
}

.markdown-body :deep(li) {
  margin-bottom: var(--space-1);
}

.markdown-body :deep(li > ul),
.markdown-body :deep(li > ol) {
  margin-bottom: 0;
}

.markdown-body :deep(blockquote) {
  border-left: 3px solid var(--color-accent);
  padding: var(--space-2) var(--space-4);
  margin: 0 0 var(--space-3);
  color: var(--color-text-muted);
  background: rgba(255, 255, 255, 0.02);
  border-radius: 0 var(--radius-sm) var(--radius-sm) 0;
}

.markdown-body :deep(blockquote p:last-child) {
  margin-bottom: 0;
}

.markdown-body :deep(table) {
  width: 100%;
  border-collapse: collapse;
  margin: 0 0 var(--space-3);
  font-size: 13px;
}

.markdown-body :deep(th) {
  background: var(--color-surface-raised);
  font-weight: 600;
  color: var(--color-text-primary);
  text-align: left;
  padding: var(--space-2) var(--space-3);
  border: 1px solid var(--color-border-default);
}

.markdown-body :deep(td) {
  padding: var(--space-2) var(--space-3);
  border: 1px solid var(--color-border-default);
}

.markdown-body :deep(tr:nth-child(even)) {
  background: rgba(255, 255, 255, 0.02);
}

.markdown-body :deep(hr) {
  border: none;
  border-top: 1px solid var(--color-border-default);
  margin: var(--space-4) 0;
}

.markdown-body :deep(img) {
  max-width: 100%;
  border-radius: var(--radius-md);
}

.markdown-body :deep(input[type="checkbox"]) {
  margin-right: var(--space-2);
}

.markdown-body :deep(> *:first-child) {
  margin-top: 0;
}

.markdown-body :deep(> *:last-child) {
  margin-bottom: 0;
}
</style>
