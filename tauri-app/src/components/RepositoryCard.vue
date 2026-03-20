<script setup lang="ts">
import { ref, nextTick } from 'vue'
import { RefreshCw, X, Pencil } from 'lucide-vue-next'
import type { Repository, RepoGroup } from '../types'

const props = defineProps<{
  repo: Repository
  prCount?: number
  lastSynced?: string | null
  groups?: RepoGroup[]
}>()

const emit = defineEmits<{
  remove: [id: number]
  sync: [id: number]
  'update-branch': [id: number, branch: string]
}>()

// Inline branch editing state
const editingBranch = ref(false)
const editBranchValue = ref('')
const branchInput = ref<HTMLInputElement | null>(null)

function startEditBranch() {
  editBranchValue.value = props.repo.default_branch
  editingBranch.value = true
  nextTick(() => {
    branchInput.value?.focus()
    branchInput.value?.select()
  })
}

function cancelEditBranch() {
  editingBranch.value = false
}

function saveBranch() {
  const trimmed = editBranchValue.value.trim()
  if (trimmed && trimmed !== props.repo.default_branch) {
    emit('update-branch', props.repo.id, trimmed)
  }
  editingBranch.value = false
}

function onBranchKeydown(event: KeyboardEvent) {
  if (event.key === 'Enter') {
    saveBranch()
  } else if (event.key === 'Escape') {
    cancelEditBranch()
  }
}
</script>

<template>
  <div class="repo-card">
    <div class="repo-header">
      <div class="repo-name">
        <span class="repo-owner">{{ repo.owner }}/</span>
        <span class="repo-repo">{{ repo.name }}</span>
      </div>
      <div class="repo-actions">
        <button class="btn-action btn-sync" @click="emit('sync', repo.id)" title="Sync PRs"><RefreshCw :size="14" /></button>
        <button class="btn-action btn-remove" @click="emit('remove', repo.id)" title="Remove"><X :size="14" /></button>
      </div>
    </div>
    <div v-if="groups && groups.length > 0" class="repo-group-tags">
      <span
        v-for="group in groups"
        :key="group.id"
        class="group-tag"
        :style="{ background: group.colour + '22', color: group.colour, borderColor: group.colour + '44' }"
      >
        {{ group.name }}
      </span>
    </div>
    <div class="repo-meta">
      <span class="meta-item branch-meta">
        Branch:
        <template v-if="editingBranch">
          <input
            ref="branchInput"
            v-model="editBranchValue"
            class="branch-edit-input"
            @keydown="onBranchKeydown"
            @blur="saveBranch"
          />
        </template>
        <template v-else>
          <code class="branch-display" @click="startEditBranch">
            {{ repo.default_branch }}
            <Pencil :size="11" class="branch-edit-icon" />
          </code>
        </template>
      </span>
      <span v-if="prCount !== undefined" class="meta-item">
        {{ prCount }} open PRs
      </span>
      <span v-if="lastSynced" class="meta-item">
        Last synced: {{ lastSynced }}
      </span>
    </div>
  </div>
</template>

<style scoped>
.repo-card {
  background: var(--color-surface-panel);
  border: 1px solid var(--color-border-default);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-card);
  padding: var(--space-5);
  transition: transform var(--transition-fast), box-shadow var(--transition-fast);
}

.repo-card:hover {
  transform: scale(1.005);
  box-shadow: var(--shadow-panel);
}

.repo-card:active {
  transform: scale(0.99);
}

.repo-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: var(--space-3);
}

.repo-name {
  font-size: 15px;
}

.repo-owner {
  color: var(--color-text-secondary);
}

.repo-repo {
  font-weight: 600;
  color: var(--color-text-primary);
}

.repo-actions {
  display: flex;
  gap: var(--space-2);
}

.btn-action {
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--color-surface-raised);
  color: var(--color-text-secondary);
  border-radius: var(--radius-md);
  border: 1px solid var(--color-border-default);
  padding: 0;
  font-size: 14px;
  cursor: pointer;
  transition: all var(--transition-fast);
}

.btn-action:hover {
  background: var(--color-surface-hover);
  color: var(--color-text-primary);
  border-color: var(--color-border-hover);
}

.btn-action:active {
  transform: scale(0.97);
}

.btn-action:focus-visible {
  box-shadow: 0 0 0 2px var(--color-accent-muted);
  outline: none;
}

.btn-remove:hover {
  color: var(--color-status-danger);
  border-color: rgba(220, 38, 38, 0.3);
  background: rgba(220, 38, 38, 0.1);
}

.repo-group-tags {
  display: flex;
  flex-wrap: wrap;
  gap: var(--space-1-5);
  margin-bottom: var(--space-3);
}

.group-tag {
  font-size: 11px;
  font-weight: 500;
  padding: 1px var(--space-2);
  border-radius: var(--radius-full);
  border: 1px solid;
}

.repo-meta {
  display: flex;
  gap: var(--space-4);
  font-size: 13px;
  color: var(--color-text-muted);
}

.repo-meta code {
  font-size: 12px;
  color: var(--color-text-secondary);
  background: var(--color-surface-raised);
  padding: 1px var(--space-2);
  border-radius: var(--radius-sm);
}

/* Inline branch editing */
.branch-meta {
  display: flex;
  align-items: center;
  gap: var(--space-1);
}

.branch-display {
  cursor: pointer;
  display: inline-flex;
  align-items: center;
  gap: var(--space-1);
  transition: background var(--transition-fast), border-color var(--transition-fast);
  border: 1px solid transparent;
  padding: 1px var(--space-2);
}

.branch-display:hover {
  background: var(--color-surface-hover);
  border-color: var(--color-border-hover);
}

.branch-edit-icon {
  opacity: 0;
  color: var(--color-text-muted);
  transition: opacity var(--transition-fast);
  flex-shrink: 0;
}

.branch-display:hover .branch-edit-icon {
  opacity: 1;
}

.branch-edit-input {
  font-size: 12px;
  font-family: var(--font-mono);
  background: var(--color-surface-input);
  border: 1px solid var(--color-border-focus);
  border-radius: var(--radius-sm);
  padding: 1px var(--space-2);
  color: var(--color-text-primary);
  width: 120px;
  outline: none;
  box-shadow: 0 0 0 2px var(--color-accent-muted);
  transition: border-color var(--transition-fast), box-shadow var(--transition-fast);
}
</style>
