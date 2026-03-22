<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { Save, Trash2, Edit3 } from 'lucide-vue-next'
import { SButton, SInput, SIconButton } from '@stuntrocket/ui'
import { useFilterPresets } from '../composables/useFilterPresets'
import type { FilterPreset } from '../types'

const { presets, fetchPresets, applyPreset, saveCurrentAsPreset, deletePreset, renamePreset } = useFilterPresets()
const activePresetId = ref<number | null>(null)
const showSaveDialog = ref(false)
const newPresetName = ref('')
const editingId = ref<number | null>(null)
const editingName = ref('')

onMounted(() => {
  fetchPresets()
})

function handleApply(preset: FilterPreset) {
  activePresetId.value = preset.id
  applyPreset(preset)
}

async function handleSave() {
  const name = newPresetName.value.trim()
  if (!name) return
  await saveCurrentAsPreset(name)
  newPresetName.value = ''
  showSaveDialog.value = false
}

async function handleDelete(id: number) {
  await deletePreset(id)
  if (activePresetId.value === id) {
    activePresetId.value = null
  }
}

function startRename(preset: FilterPreset) {
  editingId.value = preset.id
  editingName.value = preset.name
}

async function finishRename() {
  if (editingId.value !== null && editingName.value.trim()) {
    await renamePreset(editingId.value, editingName.value.trim())
  }
  editingId.value = null
  editingName.value = ''
}
</script>

<template>
  <div class="filter-presets-bar">
    <span class="presets-label">Presets</span>
    <div class="presets-list">
      <button
        v-for="preset in presets"
        :key="preset.id"
        class="preset-chip"
        :class="{ active: activePresetId === preset.id }"
        @click="handleApply(preset)"
      >
        <template v-if="editingId === preset.id">
          <input
            v-model="editingName"
            class="preset-rename-input"
            @keyup.enter="finishRename"
            @blur="finishRename"
            @click.stop
          />
        </template>
        <template v-else>
          <span class="preset-name">{{ preset.name }}</span>
          <SIconButton
            v-if="!preset.is_builtin"
            variant="ghost"
            size="sm"
            tooltip="Rename"
            @click.stop="startRename(preset)"
          >
            <Edit3 :size="10" />
          </SIconButton>
          <SIconButton
            v-if="!preset.is_builtin"
            variant="danger"
            size="sm"
            tooltip="Delete"
            @click.stop="handleDelete(preset.id)"
          >
            <Trash2 :size="10" />
          </SIconButton>
        </template>
      </button>
    </div>
    <SButton
      v-if="!showSaveDialog"
      variant="ghost"
      size="sm"
      @click="showSaveDialog = true"
    >
      <Save :size="14" />
      Save
    </SButton>
    <div v-if="showSaveDialog" class="preset-save-dialog">
      <SInput
        v-model="newPresetName"
        placeholder="Preset name..."
        @keyup.enter="handleSave"
      />
      <SButton variant="primary" size="sm" @click="handleSave">Save</SButton>
      <SButton variant="ghost" size="sm" @click="showSaveDialog = false">&times;</SButton>
    </div>
  </div>
</template>

<style scoped>
.filter-presets-bar {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  flex-wrap: wrap;
}

.presets-label {
  font-size: 12px;
  font-weight: 600;
  color: var(--color-text-muted);
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.presets-list {
  display: flex;
  gap: var(--space-1);
  flex-wrap: wrap;
}

.preset-chip {
  display: inline-flex;
  align-items: center;
  gap: var(--space-1);
  padding: var(--space-1) var(--space-3);
  background: var(--color-surface-raised);
  border: 1px solid transparent;
  border-radius: var(--radius-full);
  font-size: 12px;
  color: var(--color-text-secondary);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.preset-chip:hover {
  background: var(--color-surface-hover);
  border-color: var(--color-border-hover);
}

.preset-chip.active {
  background: var(--color-accent-muted);
  color: var(--color-accent);
  border-color: rgba(20, 184, 166, 0.3);
  font-weight: 600;
}

.preset-name {
  white-space: nowrap;
}

.preset-rename-input {
  width: 80px;
  background: var(--color-surface-input);
  border: 1px solid var(--color-border-focus);
  border-radius: var(--radius-sm);
  padding: 1px var(--space-1);
  font-size: 12px;
  color: var(--color-text-primary);
  outline: none;
}

.preset-save-dialog {
  display: flex;
  align-items: center;
  gap: var(--space-1);
}
</style>
