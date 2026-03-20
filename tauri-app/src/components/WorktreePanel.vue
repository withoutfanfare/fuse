<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { AlertTriangle } from 'lucide-vue-next'
import { invoke } from '@tauri-apps/api/core'
import { useGrove } from '../composables/useGrove'
import { useConfirm } from '@stuntrocket/ui'
import { useToastStore } from '../stores/toast'

const props = defineProps<{
  repoName: string
  branch: string
  baseBranch?: string
}>()

const emit = defineEmits<{
  (e: 'review-requested', worktreePath: string): void
}>()

const { worktrees, loading, error, listWorktrees, addWorktree, removeWorktree } = useGrove()
const toastStore = useToastStore()
const { confirm } = useConfirm()
const reviewRequested = ref(false)

onMounted(() => {
  listWorktrees(props.repoName)
})

const branchWorktree = computed(() =>
  worktrees.value.find(w => w.branch === props.branch)
)

const branchHasWorktree = computed(() => !!branchWorktree.value)

/** PRs targeting main or master should never be merged — warn the user. */
const isForbiddenTarget = computed(() => {
  if (!props.baseBranch) return false
  const base = props.baseBranch.toLowerCase()
  return base === 'main' || base === 'master'
})

async function handleCreate() {
  await addWorktree(props.repoName, props.branch)
}

async function handleRemove() {
  const confirmed = await confirm({
    title: 'Remove Worktree',
    message: `Remove worktree for branch ${props.branch}?`,
    confirmLabel: 'Remove',
    danger: true,
  })
  if (!confirmed) return
  reviewRequested.value = false
  await removeWorktree(props.repoName, props.branch)
}

function handleStartReview() {
  reviewRequested.value = true
  if (branchWorktree.value?.path) {
    emit('review-requested', branchWorktree.value.path)
  }
}

/** Copy the claude review command to clipboard */
async function copyReviewCommand() {
  const cmd = `/worktree-review ${props.repoName} ${props.branch} origin/develop`
  await navigator.clipboard.writeText(cmd)
  toastStore.addToast('info', 'Copied to clipboard', 'Review command ready to paste')
}

/** Open the worktree directory in the configured editor */
async function openInEditor() {
  if (!branchWorktree.value?.path) return
  try {
    await invoke('open_in_editor', { path: branchWorktree.value.path })
    toastStore.addToast('success', 'Editor opened', 'Worktree opened in your editor')
  } catch (e) {
    toastStore.addToast('error', 'Failed to open editor', String(e))
  }
}
</script>

<template>
  <div class="worktree-panel">
    <div class="panel-header">
      <h3 class="panel-title">Review Worktree</h3>
      <span v-if="loading" class="loading-indicator">Loading…</span>
    </div>

    <!-- Merge protection warning -->
    <div v-if="isForbiddenTarget" class="merge-warning">
      <AlertTriangle :size="14" class="warning-icon" />
      <div class="warning-text">
        <strong>Dangerous target branch</strong>
        <p>This PR targets <code>{{ baseBranch }}</code>. PRs should only merge into <code>staging</code> — never main or master.</p>
      </div>
    </div>

    <div v-if="error" class="panel-error">{{ error }}</div>

    <div class="worktree-actions">
      <template v-if="!branchHasWorktree">
        <button
          class="btn-create"
          :disabled="loading"
          @click="handleCreate"
        >
          Create Review Worktree
        </button>
        <p class="action-hint">Creates worktree based on <code>origin/develop</code></p>
      </template>
      <template v-else>
        <div class="active-worktree">
          <div class="wt-status">
            <span class="status-dot"></span>
            <span class="status-text">Worktree active</span>
          </div>
          <div v-if="branchWorktree?.path" class="wt-path-display">
            <code>{{ branchWorktree.path }}</code>
          </div>
        </div>

        <div class="review-actions">
          <button
            class="btn-review"
            :disabled="loading || reviewRequested"
            @click="handleStartReview"
          >
            {{ reviewRequested ? 'Review Requested' : 'Start Code Review' }}
          </button>
          <button
            class="btn-open-editor"
            title="Open worktree in configured editor"
            @click="openInEditor"
          >
            Open in Editor
          </button>
          <button
            class="btn-copy-cmd"
            title="Copy Claude review command to clipboard"
            @click="copyReviewCommand"
          >
            Copy Review Command
          </button>
        </div>

        <button
          class="btn-remove"
          :disabled="loading"
          @click="handleRemove"
        >
          Remove Worktree
        </button>
      </template>
    </div>

    <div v-if="!branchHasWorktree && !loading && worktrees.length === 0" class="empty-state">
      No worktrees found for {{ repoName }}
    </div>
  </div>
</template>

<style scoped>
.worktree-panel {
  background: var(--color-surface-panel);
  border: 1px solid var(--color-border-default);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-card);
  padding: var(--space-5);
}

.panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: var(--space-4);
}

.panel-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--color-text-primary);
}

.loading-indicator {
  font-size: 12px;
  color: var(--color-text-muted);
}

.merge-warning {
  display: flex;
  gap: var(--space-3);
  align-items: flex-start;
  background: rgba(234, 179, 8, 0.1);
  border: 1px solid rgba(234, 179, 8, 0.3);
  border-radius: var(--radius-md);
  padding: var(--space-3) var(--space-4);
  margin-bottom: var(--space-4);
}

.warning-icon {
  font-size: 16px;
  flex-shrink: 0;
  margin-top: 1px;
}

.warning-text {
  font-size: 13px;
  color: var(--color-text-secondary);
  line-height: 1.4;
}

.warning-text strong {
  display: block;
  color: var(--color-status-warning);
  font-size: 12px;
  text-transform: uppercase;
  letter-spacing: 0.03em;
  margin-bottom: var(--space-1);
}

.warning-text p {
  margin: 0;
}

.warning-text code {
  background: rgba(234, 179, 8, 0.15);
  padding: 1px var(--space-1);
  border-radius: var(--radius-sm);
  font-size: 12px;
}

.panel-error {
  background: rgba(220, 38, 38, 0.1);
  color: var(--color-status-danger);
  padding: var(--space-3);
  border-radius: var(--radius-sm);
  font-size: 13px;
  margin-bottom: var(--space-4);
  border: 1px solid rgba(220, 38, 38, 0.2);
}

.worktree-actions {
  margin-bottom: var(--space-4);
  display: flex;
  flex-direction: column;
  gap: var(--space-3);
}

.action-hint {
  font-size: 11px;
  color: var(--color-text-muted);
  margin: 0;
  text-align: center;
}

.action-hint code {
  background: var(--color-surface-raised);
  padding: 1px var(--space-1);
  border-radius: var(--radius-sm);
}

.active-worktree {
  background: rgba(34, 197, 94, 0.08);
  border: 1px solid rgba(34, 197, 94, 0.2);
  border-radius: var(--radius-md);
  padding: var(--space-3);
}

.wt-status {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  margin-bottom: var(--space-2);
}

.status-dot {
  width: 8px;
  height: 8px;
  border-radius: var(--radius-full);
  background: var(--color-status-success);
  box-shadow: 0 0 6px rgba(34, 197, 94, 0.4);
}

.status-text {
  font-size: 12px;
  font-weight: 600;
  color: var(--color-status-success);
  text-transform: uppercase;
  letter-spacing: 0.03em;
}

.wt-path-display code {
  font-size: 11px;
  color: var(--color-text-muted);
  font-family: var(--font-mono);
  word-break: break-all;
}

.review-actions {
  display: flex;
  gap: var(--space-2);
}

.btn-review {
  flex: 1;
  background: rgba(139, 92, 246, 0.2);
  color: #a78bfa;
  font-weight: 600;
  padding: var(--space-3);
  border-radius: var(--radius-md);
  border: 1px solid rgba(139, 92, 246, 0.3);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.btn-review:hover:not(:disabled) {
  background: rgba(139, 92, 246, 0.3);
  border-color: rgba(139, 92, 246, 0.5);
}

.btn-review:active:not(:disabled) {
  transform: scale(0.97);
}

.btn-review:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn-review:focus-visible {
  box-shadow: 0 0 0 2px var(--color-accent-muted);
  outline: none;
}

.btn-open-editor {
  background: rgba(59, 130, 246, 0.15);
  color: var(--color-status-info);
  font-size: 12px;
  padding: var(--space-2) var(--space-3);
  border-radius: var(--radius-md);
  border: 1px solid rgba(59, 130, 246, 0.25);
  cursor: pointer;
  transition: all var(--transition-fast);
  white-space: nowrap;
}

.btn-open-editor:hover {
  background: rgba(59, 130, 246, 0.25);
  border-color: rgba(59, 130, 246, 0.4);
}

.btn-open-editor:active {
  transform: scale(0.97);
}

.btn-open-editor:focus-visible {
  box-shadow: 0 0 0 2px var(--color-accent-muted);
  outline: none;
}

.btn-copy-cmd {
  background: var(--color-surface-raised);
  color: var(--color-text-secondary);
  font-size: 12px;
  padding: var(--space-2) var(--space-3);
  border-radius: var(--radius-md);
  border: 1px solid var(--color-border-default);
  cursor: pointer;
  transition: all var(--transition-fast);
  white-space: nowrap;
}

.btn-copy-cmd:hover {
  background: var(--color-surface-hover);
  border-color: var(--color-border-hover);
}

.btn-copy-cmd:active {
  transform: scale(0.97);
}

.btn-copy-cmd:focus-visible {
  box-shadow: 0 0 0 2px var(--color-accent-muted);
  outline: none;
}

.btn-create {
  background: rgba(59, 130, 246, 0.2);
  color: var(--color-status-info);
  font-weight: 600;
  width: 100%;
  padding: var(--space-3);
  border-radius: var(--radius-md);
  border: 1px solid rgba(59, 130, 246, 0.3);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.btn-create:hover:not(:disabled) {
  background: rgba(59, 130, 246, 0.3);
  border-color: rgba(59, 130, 246, 0.5);
}

.btn-create:active:not(:disabled) {
  transform: scale(0.97);
}

.btn-create:focus-visible {
  box-shadow: 0 0 0 2px var(--color-accent-muted);
  outline: none;
}

.btn-remove {
  background: rgba(220, 38, 38, 0.1);
  color: var(--color-status-danger);
  border: 1px solid rgba(220, 38, 38, 0.2);
  font-weight: 500;
  font-size: 12px;
  width: 100%;
  padding: var(--space-2);
  border-radius: var(--radius-md);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.btn-remove:hover:not(:disabled) {
  background: rgba(220, 38, 38, 0.2);
  border-color: rgba(220, 38, 38, 0.4);
}

.btn-remove:active:not(:disabled) {
  transform: scale(0.97);
}

.btn-remove:focus-visible {
  box-shadow: 0 0 0 2px var(--color-accent-muted);
  outline: none;
}

.empty-state {
  color: var(--color-text-muted);
  font-size: 13px;
  text-align: center;
  padding: var(--space-4);
}
</style>
