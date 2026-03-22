<script setup lang="ts">
import { ref } from 'vue'
import { Copy, Send, FileText, X } from 'lucide-vue-next'
import { SButton, SCard } from '@stuntrocket/ui'
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
    <SButton
      v-if="!showPreview"
      variant="primary"
      size="sm"
      @click="handleGenerate"
    >
      <FileText :size="14" />
      Generate Summary
    </SButton>

    <SCard v-if="showPreview && summaryMarkdown" variant="content">
      <div class="summary-header">
        <h3 class="summary-title">Review Summary Preview</h3>
        <div class="summary-actions">
          <SButton variant="ghost" size="sm" @click="handleCopy">
            <Copy :size="12" />
            Copy
          </SButton>
          <SButton
            variant="primary"
            size="sm"
            :disabled="posting"
            :loading="posting"
            @click="handlePost"
          >
            <Send :size="12" />
            {{ posting ? 'Posting...' : 'Post to GitHub' }}
          </SButton>
          <SButton variant="ghost" size="sm" @click="showPreview = false">
            <X :size="14" />
          </SButton>
        </div>
      </div>
      <div class="summary-content">
        <MarkdownRenderer :content="summaryMarkdown" />
      </div>
    </SCard>
  </div>
</template>

<style scoped>
.review-summary-panel {
  margin-top: var(--space-3);
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

.summary-content {
  padding: var(--space-4);
  max-height: 400px;
  overflow-y: auto;
}
</style>
