<script setup lang="ts">
import { ref, computed } from 'vue'
import { Copy, Send, FileText } from 'lucide-vue-next'
import { useReviewSummary } from '../composables/useReviewSummary'
import { useToastStore } from '../stores/toast'
import MarkdownRenderer from './MarkdownRenderer.vue'
import type { PullRequest, Bookmark } from '../types'

const props = defineProps<{
  pr: PullRequest
  checkedRules: Record<number, boolean>
  reviewRules: string[]
  bookmarks: Bookmark[]
  reviewElapsedSeconds: number
  filesReviewed?: number
  totalFiles?: number
}>()

const toast = useToastStore()
const { summaryMarkdown, posting, postError, generateSummary, copyToClipboard, postToGitHub } = useReviewSummary()
const showPreview = ref(false)

function handleGenerate() {
  generateSummary({
    pr: props.pr,
    checkedRules: props.checkedRules,
    reviewRules: props.reviewRules,
    bookmarks: props.bookmarks,
    reviewElapsedSeconds: props.reviewElapsedSeconds,
    filesReviewed: props.filesReviewed,
    totalFiles: props.totalFiles,
  })
  showPreview.value = true
}

async function handleCopy() {
  const ok = await copyToClipboard()
  if (ok) {
    toast.addToast('success', 'Copied', 'Review summary copied to clipboard')
  } else {
    toast.addToast('error', 'Copy failed', 'Could not copy to clipboard')
  }
}

async function handlePost() {
  const ok = await postToGitHub(props.pr.id)
  if (ok) {
    toast.addToast('success', 'Posted', `Review summary posted to PR #${props.pr.number}`)
    showPreview.value = false
  } else {
    toast.addToast('error', 'Post failed', postError.value ?? 'Failed to post comment')
  }
}
</script>

<template>
  <div class="review-summary-panel">
    <button
      v-if="!showPreview"
      class="btn-generate"
      @click="handleGenerate"
    >
      <FileText :size="14" />
      Generate Summary
    </button>

    <div v-if="showPreview && summaryMarkdown" class="summary-preview">
      <div class="summary-header">
        <h3 class="summary-title">Review Summary Preview</h3>
        <div class="summary-actions">
          <button class="btn-action" @click="handleCopy">
            <Copy :size="12" />
            Copy
          </button>
          <button
            class="btn-action btn-post"
            :disabled="posting"
            @click="handlePost"
          >
            <Send :size="12" />
            {{ posting ? 'Posting…' : 'Post to GitHub' }}
          </button>
          <button class="btn-close" @click="showPreview = false">&times;</button>
        </div>
      </div>
      <div class="summary-content">
        <MarkdownRenderer :content="summaryMarkdown" />
      </div>
    </div>
  </div>
</template>

<style scoped>
.review-summary-panel {
  margin-top: var(--space-3);
}

.btn-generate {
  display: inline-flex;
  align-items: center;
  gap: var(--space-2);
  padding: var(--space-2) var(--space-4);
  background: rgba(59, 130, 246, 0.15);
  color: var(--color-status-info);
  font-weight: 500;
  font-size: 13px;
  border: 1px solid rgba(59, 130, 246, 0.25);
  border-radius: var(--radius-md);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.btn-generate:hover {
  background: rgba(59, 130, 246, 0.25);
  border-color: rgba(59, 130, 246, 0.4);
}

.summary-preview {
  background: var(--color-surface-panel);
  border: 1px solid var(--color-border-default);
  border-radius: var(--radius-lg);
  overflow: hidden;
}

.summary-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: var(--space-3) var(--space-4);
  border-bottom: 1px solid var(--color-border-default);
}

.summary-title {
  font-size: 13px;
  font-weight: 600;
  color: var(--color-text-primary);
}

.summary-actions {
  display: flex;
  gap: var(--space-2);
  align-items: center;
}

.btn-action {
  display: inline-flex;
  align-items: center;
  gap: var(--space-1);
  padding: var(--space-1) var(--space-2);
  background: var(--color-surface-raised);
  border: 1px solid var(--color-border-default);
  border-radius: var(--radius-md);
  font-size: 11px;
  font-weight: 600;
  color: var(--color-text-secondary);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.btn-action:hover {
  background: var(--color-surface-hover);
  border-color: var(--color-border-hover);
}

.btn-post {
  background: rgba(34, 197, 94, 0.15);
  color: var(--color-status-success);
  border-color: rgba(34, 197, 94, 0.3);
}

.btn-post:hover:not(:disabled) {
  background: rgba(34, 197, 94, 0.25);
}

.btn-post:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn-close {
  background: none;
  border: none;
  color: var(--color-text-muted);
  font-size: 18px;
  cursor: pointer;
  padding: 0 var(--space-1);
}

.btn-close:hover {
  color: var(--color-text-primary);
}

.summary-content {
  padding: var(--space-4);
  max-height: 400px;
  overflow-y: auto;
}
</style>
