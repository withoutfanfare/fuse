<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { Plus, Trash2, GripVertical } from 'lucide-vue-next'
import { useToastStore } from '../stores/toast'
import type { ReviewTemplate } from '../types'

const toastStore = useToastStore()
const templates = ref<ReviewTemplate[]>([])
const saving = ref(false)
const saved = ref(false)

onMounted(async () => {
  await loadTemplates()
})

async function loadTemplates() {
  try {
    templates.value = await invoke<ReviewTemplate[]>('list_templates')
  } catch {
    templates.value = []
  }
}

function addTemplate() {
  const position = templates.value.length
  templates.value.push({
    id: 0,
    name: '',
    body: '',
    position,
  })
}

function removeTemplate(index: number) {
  templates.value.splice(index, 1)
  // Re-index positions
  templates.value.forEach((t, i) => { t.position = i })
}

function moveUp(index: number) {
  if (index <= 0) return
  const temp = templates.value[index]
  templates.value[index] = templates.value[index - 1]
  templates.value[index - 1] = temp
  templates.value.forEach((t, i) => { t.position = i })
}

function moveDown(index: number) {
  if (index >= templates.value.length - 1) return
  const temp = templates.value[index]
  templates.value[index] = templates.value[index + 1]
  templates.value[index + 1] = temp
  templates.value.forEach((t, i) => { t.position = i })
}

async function saveTemplates() {
  saving.value = true
  try {
    await invoke('set_templates', { templates: templates.value })
    saved.value = true
    toastStore.addToast('success', 'Templates saved', 'Review templates updated successfully')
    setTimeout(() => { saved.value = false }, 2000)
    await loadTemplates()
  } catch (e) {
    toastStore.addToast('error', 'Save failed', String(e))
  } finally {
    saving.value = false
  }
}
</script>

<template>
  <div class="template-manager">
    <div class="manager-header">
      <h3 class="manager-title">Review Templates</h3>
      <p class="manager-description">Quick review messages for approving pull requests</p>
    </div>

    <div class="templates-list">
      <div
        v-for="(tpl, index) in templates"
        :key="index"
        class="template-item"
      >
        <div class="template-drag">
          <button
            class="btn-reorder"
            :disabled="index === 0"
            @click="moveUp(index)"
            title="Move up"
          >
            <GripVertical :size="14" />
          </button>
        </div>
        <div class="template-fields">
          <input
            v-model="tpl.name"
            type="text"
            class="input-field template-name-input"
            placeholder="Template name"
          />
          <textarea
            v-model="tpl.body"
            class="input-field template-body-input"
            placeholder="Review message body"
            rows="2"
          />
        </div>
        <div class="template-actions-col">
          <button class="btn-reorder" :disabled="index === 0" @click="moveUp(index)" title="Move up">^</button>
          <button class="btn-reorder" :disabled="index === templates.length - 1" @click="moveDown(index)" title="Move down">v</button>
          <button class="btn-remove" @click="removeTemplate(index)" title="Remove template">
            <Trash2 :size="14" />
          </button>
        </div>
      </div>

      <div v-if="templates.length === 0" class="templates-empty">
        No review templates configured. Add one to get started.
      </div>
    </div>

    <div class="manager-footer">
      <button class="btn-add" @click="addTemplate">
        <Plus :size="14" />
        Add Template
      </button>
      <button class="btn-save" :disabled="saving" @click="saveTemplates">
        {{ saved ? 'Saved!' : saving ? 'Saving...' : 'Save Templates' }}
      </button>
    </div>
  </div>
</template>

<style scoped>
.template-manager {
  background: var(--color-surface-panel);
  border: 1px solid var(--color-border-default);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-card);
  overflow: hidden;
}

.manager-header {
  padding: var(--space-5);
  border-bottom: 1px solid rgba(255, 255, 255, 0.04);
}

.manager-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--color-text-primary);
  margin-bottom: var(--space-1);
}

.manager-description {
  font-size: 13px;
  color: var(--color-text-muted);
}

.templates-list {
  padding: var(--space-4) var(--space-5);
}

.template-item {
  display: flex;
  gap: var(--space-3);
  align-items: flex-start;
  padding: var(--space-3) 0;
  border-bottom: 1px solid rgba(255, 255, 255, 0.04);
}

.template-item:last-child {
  border-bottom: none;
}

.template-drag {
  display: flex;
  align-items: center;
  padding-top: var(--space-2);
  color: var(--color-text-muted);
  cursor: grab;
}

.template-fields {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
}

.input-field {
  background: var(--color-surface-input);
  border: 1px solid var(--color-border-default);
  border-radius: var(--radius-md);
  padding: var(--space-2) var(--space-3);
  color: var(--color-text-primary);
  font-size: 13px;
  transition: border-color var(--transition-fast);
  width: 100%;
}

.input-field:focus {
  border-color: var(--color-border-focus);
  box-shadow: 0 0 0 2px var(--color-accent-muted);
  outline: none;
}

.template-name-input {
  font-weight: 600;
}

.template-body-input {
  font-family: var(--font-sans);
  resize: vertical;
  min-height: 48px;
}

.template-actions-col {
  display: flex;
  flex-direction: column;
  gap: var(--space-1);
  padding-top: var(--space-1);
}

.btn-reorder {
  background: none;
  border: none;
  color: var(--color-text-muted);
  font-size: 12px;
  padding: var(--space-0-5);
  cursor: pointer;
  border-radius: var(--radius-sm);
  transition: all var(--transition-fast);
  line-height: 1;
}

.btn-reorder:hover:not(:disabled) {
  color: var(--color-text-primary);
  background: var(--color-surface-hover);
}

.btn-reorder:disabled {
  opacity: 0.3;
  cursor: not-allowed;
}

.btn-remove {
  background: none;
  border: none;
  color: var(--color-text-muted);
  padding: var(--space-0-5);
  cursor: pointer;
  border-radius: var(--radius-sm);
  transition: all var(--transition-fast);
}

.btn-remove:hover {
  color: var(--color-status-danger);
  background: rgba(220, 38, 38, 0.1);
}

.templates-empty {
  text-align: center;
  padding: var(--space-6);
  color: var(--color-text-muted);
  font-size: 13px;
}

.manager-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: var(--space-4) var(--space-5);
  border-top: 1px solid rgba(255, 255, 255, 0.04);
}

.btn-add {
  display: inline-flex;
  align-items: center;
  gap: var(--space-1);
  background: var(--color-surface-raised);
  color: var(--color-text-secondary);
  font-size: 13px;
  font-weight: 500;
  padding: var(--space-2) var(--space-3);
  border-radius: var(--radius-md);
  border: 1px solid var(--color-border-default);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.btn-add:hover {
  background: var(--color-surface-hover);
  color: var(--color-text-primary);
  border-color: var(--color-border-hover);
}

.btn-save {
  background: var(--color-accent);
  color: var(--color-text-inverse);
  font-weight: 600;
  font-size: 13px;
  padding: var(--space-2) var(--space-5);
  border-radius: var(--radius-md);
  border: none;
  cursor: pointer;
  transition: background var(--transition-fast), transform var(--transition-fast);
}

.btn-save:hover:not(:disabled) {
  background: var(--color-accent-hover);
}

.btn-save:active:not(:disabled) {
  transform: scale(0.97);
}

.btn-save:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.btn-save:focus-visible {
  box-shadow: 0 0 0 2px var(--color-accent-muted);
  outline: none;
}
</style>
