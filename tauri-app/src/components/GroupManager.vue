<script setup lang="ts">
import { ref } from 'vue'
import { X, Plus } from 'lucide-vue-next'
import { useGroupsStore } from '../stores/groups'
import { useRepositoriesStore } from '../stores/repositories'
import { useConfirm } from '@stuntrocket/ui'

const groupsStore = useGroupsStore()
const repoStore = useRepositoriesStore()
const { confirm } = useConfirm()

const newGroupName = ref('')
const selectedColour = ref('#ff6b35')
const creating = ref(false)

const palette = [
  '#ff6b35', '#e74c3c', '#9b59b6', '#8b5cf6',
  '#3b82f6', '#14b8a6', '#22c55e', '#eab308',
  '#f97316', '#ec4899', '#6366f1', '#64748b',
]

async function createGroup() {
  if (!newGroupName.value.trim()) return
  creating.value = true
  await groupsStore.create(newGroupName.value.trim(), selectedColour.value)
  newGroupName.value = ''
  selectedColour.value = '#ff6b35'
  creating.value = false
}

async function deleteGroup(id: number) {
  const group = groupsStore.groups.find(g => g.id === id)
  const confirmed = await confirm({
    title: 'Delete Group',
    message: `Delete the group "${group?.name ?? ''}"? Repositories will not be removed.`,
    confirmLabel: 'Delete',
    danger: true,
  })
  if (!confirmed) return
  await groupsStore.remove(id)
}

async function toggleRepo(groupId: number, repoId: number) {
  const group = groupsStore.groups.find(g => g.id === groupId)
  if (!group) return
  if (group.repo_ids.includes(repoId)) {
    await groupsStore.removeRepo(groupId, repoId)
  } else {
    await groupsStore.addRepo(groupId, repoId)
  }
}
</script>

<template>
  <div class="group-manager">
    <h2 class="section-title">Repository Groups</h2>

    <form class="create-group-form" @submit.prevent="createGroup">
      <input
        v-model="newGroupName"
        placeholder="New group name"
        class="input-field"
        required
      />
      <div class="colour-picker">
        <button
          v-for="c in palette"
          :key="c"
          type="button"
          class="colour-swatch"
          :class="{ selected: selectedColour === c }"
          :style="{ background: c }"
          @click="selectedColour = c"
        />
      </div>
      <button type="submit" class="btn-add" :disabled="creating">
        <Plus :size="14" />
        {{ creating ? 'Creating...' : 'Create' }}
      </button>
    </form>

    <div v-if="groupsStore.groups.length === 0" class="empty-groups">
      No groups yet. Create one above to organise your repositories.
    </div>

    <div v-else class="groups-list">
      <div
        v-for="group in groupsStore.groups"
        :key="group.id"
        class="group-card"
      >
        <div class="group-header">
          <div class="group-name-row">
            <span class="group-colour-dot" :style="{ background: group.colour }" />
            <span class="group-name">{{ group.name }}</span>
            <span class="group-count">{{ group.repo_ids.length }} repos</span>
          </div>
          <button class="btn-delete" @click="deleteGroup(group.id)" title="Delete group">
            <X :size="14" />
          </button>
        </div>
        <div class="group-repos">
          <button
            v-for="repo in repoStore.repos"
            :key="repo.id"
            class="repo-chip"
            :class="{ active: group.repo_ids.includes(repo.id) }"
            @click="toggleRepo(group.id, repo.id)"
          >
            {{ repo.owner }}/{{ repo.name }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.group-manager {
  background: var(--color-surface-panel);
  border: 1px solid var(--color-border-default);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-card);
  padding: var(--space-5);
}

.section-title {
  font-size: 16px;
  font-weight: 600;
  margin-bottom: var(--space-4);
  color: var(--color-text-primary);
}

.create-group-form {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  margin-bottom: var(--space-5);
  flex-wrap: wrap;
}

.input-field {
  flex: 1;
  min-width: 180px;
  background: var(--color-surface-input);
  border: 1px solid var(--color-border-default);
  border-radius: var(--radius-md);
  padding: var(--space-2) var(--space-3);
  color: var(--color-text-primary);
  font-size: 13px;
  transition: border-color var(--transition-fast);
}

.input-field:focus {
  border-color: var(--color-border-focus);
  box-shadow: 0 0 0 2px var(--color-accent-muted);
  outline: none;
}

.input-field::placeholder {
  color: var(--color-text-muted);
}

.colour-picker {
  display: flex;
  gap: var(--space-1);
  flex-wrap: wrap;
}

.colour-swatch {
  width: 20px;
  height: 20px;
  border-radius: var(--radius-full);
  border: 2px solid transparent;
  cursor: pointer;
  transition: all var(--transition-fast);
  padding: 0;
}

.colour-swatch:hover {
  transform: scale(1.15);
}

.colour-swatch.selected {
  border-color: var(--color-text-primary);
  box-shadow: 0 0 0 2px var(--color-surface-base);
}

.btn-add {
  display: inline-flex;
  align-items: center;
  gap: var(--space-1);
  background: var(--color-accent);
  color: var(--color-text-inverse);
  font-weight: 600;
  padding: var(--space-2) var(--space-4);
  white-space: nowrap;
  border-radius: var(--radius-md);
  border: none;
  cursor: pointer;
  font-size: 13px;
  transition: background var(--transition-fast), transform var(--transition-fast);
}

.btn-add:hover:not(:disabled) {
  background: var(--color-accent-hover);
}

.btn-add:active:not(:disabled) {
  transform: scale(0.97);
}

.btn-add:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.empty-groups {
  color: var(--color-text-muted);
  font-size: 13px;
  padding: var(--space-4) 0;
}

.groups-list {
  display: flex;
  flex-direction: column;
  gap: var(--space-4);
}

.group-card {
  background: var(--color-surface-raised);
  border: 1px solid var(--color-border-default);
  border-radius: var(--radius-md);
  padding: var(--space-4);
}

.group-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: var(--space-3);
}

.group-name-row {
  display: flex;
  align-items: center;
  gap: var(--space-2);
}

.group-colour-dot {
  width: 12px;
  height: 12px;
  border-radius: var(--radius-full);
  flex-shrink: 0;
}

.group-name {
  font-weight: 600;
  font-size: 14px;
  color: var(--color-text-primary);
}

.group-count {
  font-size: 12px;
  color: var(--color-text-muted);
}

.btn-delete {
  width: 28px;
  height: 28px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: none;
  border: 1px solid transparent;
  border-radius: var(--radius-md);
  color: var(--color-text-muted);
  cursor: pointer;
  padding: 0;
  transition: all var(--transition-fast);
}

.btn-delete:hover {
  color: var(--color-status-danger);
  background: rgba(220, 38, 38, 0.1);
  border-color: rgba(220, 38, 38, 0.3);
}

.group-repos {
  display: flex;
  flex-wrap: wrap;
  gap: var(--space-2);
}

.repo-chip {
  font-size: 12px;
  padding: var(--space-1) var(--space-3);
  border-radius: var(--radius-full);
  border: 1px solid var(--color-border-default);
  background: var(--color-surface-panel);
  color: var(--color-text-secondary);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.repo-chip:hover {
  border-color: var(--color-border-hover);
  color: var(--color-text-primary);
}

.repo-chip.active {
  background: var(--color-accent-muted);
  color: var(--color-accent);
  border-color: rgba(20, 184, 166, 0.3);
  font-weight: 500;
}
</style>
