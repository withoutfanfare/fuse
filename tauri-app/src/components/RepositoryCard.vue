<script setup lang="ts">
import { ref, nextTick } from 'vue'
import { SCard, SIconButton, SInput } from '@stuntrocket/ui'
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
const branchInput = ref<InstanceType<typeof SInput> | null>(null)

function startEditBranch() {
  editBranchValue.value = props.repo.default_branch
  editingBranch.value = true
  nextTick(() => {
    const input = branchInput.value?.$el?.querySelector('input') as HTMLInputElement | null
    input?.focus()
    input?.select()
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
  <SCard variant="content" hoverable>
    <div class="repo-header">
      <div class="repo-name">
        <span class="repo-owner">{{ repo.owner }}/</span>
        <span class="repo-repo">{{ repo.name }}</span>
      </div>
      <div class="repo-actions">
        <SIconButton variant="ghost" size="sm" tooltip="Sync PRs" @click="emit('sync', repo.id)">
          <RefreshCw :size="14" />
        </SIconButton>
        <SIconButton variant="danger" size="sm" tooltip="Remove" @click="emit('remove', repo.id)">
          <X :size="14" />
        </SIconButton>
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
          <SInput
            ref="branchInput"
            v-model="editBranchValue"
            size="sm"
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
  </SCard>
</template>

<style scoped>
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
  width: 120px;
}
</style>
