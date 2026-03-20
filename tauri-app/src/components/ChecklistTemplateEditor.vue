<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { Plus, Trash2, GripVertical } from 'lucide-vue-next'
import { useChecklistTemplates } from '../composables/useChecklistTemplates'
import type { ChecklistTemplate } from '../types'
import { useToastStore } from '../stores/toast'

const props = defineProps<{
  repoId: number
}>()

const { templates, loading, fetchTemplates, createTemplate, updateTemplate, deleteTemplate } = useChecklistTemplates()
const toast = useToastStore()

const editingTemplate = ref<ChecklistTemplate | null>(null)
const editName = ref('')
const editItems = ref<Array<{ text: string; description: string }>>([])
const showCreateForm = ref(false)

onMounted(() => {
  fetchTemplates(props.repoId)
})

function startCreate() {
  showCreateForm.value = true
  editName.value = 'Review Checklist'
  editItems.value = [
    { text: 'Security review completed', description: 'Check for injection vulnerabilities, auth issues, and data exposure' },
    { text: 'Tests added or updated', description: 'Ensure adequate test coverage for new and changed functionality' },
    { text: 'Documentation updated', description: 'Update API docs, README, or inline comments as needed' },
    { text: 'Performance impact assessed', description: 'Check for N+1 queries, unnecessary re-renders, or memory leaks' },
  ]
}

function startEdit(template: ChecklistTemplate) {
  editingTemplate.value = template
  editName.value = template.name
  editItems.value = template.items.map(i => ({
    text: i.text,
    description: i.description ?? '',
  }))
}

function addItem() {
  editItems.value.push({ text: '', description: '' })
}

function removeItem(index: number) {
  editItems.value.splice(index, 1)
}

async function saveCreate() {
  const items = editItems.value
    .filter(i => i.text.trim())
    .map(i => ({ text: i.text.trim(), description: i.description.trim() || undefined }))
  if (!editName.value.trim() || items.length === 0) return

  const result = await createTemplate(editName.value.trim(), items, props.repoId)
  if (result) {
    toast.addToast('success', 'Template created', `"${editName.value}" created with ${items.length} items`)
    showCreateForm.value = false
    editName.value = ''
    editItems.value = []
  }
}

async function saveEdit() {
  if (!editingTemplate.value) return
  const items = editItems.value
    .filter(i => i.text.trim())
    .map(i => ({ text: i.text.trim(), description: i.description.trim() || undefined }))

  await updateTemplate(editingTemplate.value.id, editName.value.trim(), items, props.repoId)
  toast.addToast('success', 'Template updated', `"${editName.value}" updated`)
  editingTemplate.value = null
}

async function handleDelete(template: ChecklistTemplate) {
  await deleteTemplate(template.id, props.repoId)
  toast.addToast('info', 'Template deleted', `"${template.name}" removed`)
}

function cancelEdit() {
  editingTemplate.value = null
  showCreateForm.value = false
}
</script>

<template>
  <div class="checklist-template-editor">
    <div class="editor-header">
      <h3 class="editor-title">Review Checklist Templates</h3>
      <button v-if="!showCreateForm && !editingTemplate" class="btn-create" @click="startCreate">
        <Plus :size="14" />
        New Template
      </button>
    </div>

    <div v-if="loading" class="loading-state">Loading templates...</div>

    <!-- Create / Edit form -->
    <div v-if="showCreateForm || editingTemplate" class="template-form">
      <input
        v-model="editName"
        class="template-name-input"
        placeholder="Template name..."
      />
      <div class="template-items-list">
        <div v-for="(item, idx) in editItems" :key="idx" class="template-item-row">
          <GripVertical :size="14" class="grip-icon" />
          <div class="template-item-fields">
            <input
              v-model="item.text"
              class="item-text-input"
              placeholder="Check item text..."
            />
            <input
              v-model="item.description"
              class="item-desc-input"
              placeholder="Description (optional)..."
            />
          </div>
          <button class="btn-remove-item" @click="removeItem(idx)">
            <Trash2 :size="12" />
          </button>
        </div>
      </div>
      <button class="btn-add-item" @click="addItem">
        <Plus :size="12" /> Add Item
      </button>
      <div class="form-actions">
        <button class="btn-save" @click="showCreateForm ? saveCreate() : saveEdit()">
          Save
        </button>
        <button class="btn-cancel" @click="cancelEdit">Cancel</button>
      </div>
    </div>

    <!-- Existing templates list -->
    <div v-if="!showCreateForm && !editingTemplate" class="templates-list">
      <div v-for="template in templates" :key="template.id" class="template-card">
        <div class="template-card-header">
          <span class="template-card-name">{{ template.name }}</span>
          <span class="template-card-count">{{ template.items.length }} items</span>
        </div>
        <ul class="template-card-items">
          <li v-for="item in template.items" :key="item.id" class="template-card-item">
            <span class="item-checkbox-preview">☐</span>
            <span class="item-text">{{ item.text }}</span>
          </li>
        </ul>
        <div class="template-card-actions">
          <button class="btn-edit" @click="startEdit(template)">Edit</button>
          <button class="btn-delete" @click="handleDelete(template)">Delete</button>
        </div>
      </div>

      <div v-if="!loading && templates.length === 0" class="empty-state">
        No checklist templates yet. Create one to standardise reviews for this repository.
      </div>
    </div>
  </div>
</template>

<style scoped>
.checklist-template-editor {
  background: var(--color-surface-panel);
  border: 1px solid var(--color-border-default);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-card);
  padding: var(--space-5);
}

.editor-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: var(--space-4);
}

.editor-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--color-text-primary);
}

.btn-create {
  display: inline-flex;
  align-items: center;
  gap: var(--space-1);
  padding: var(--space-1) var(--space-3);
  background: var(--color-accent-muted);
  color: var(--color-accent);
  border: 1px solid rgba(20, 184, 166, 0.3);
  border-radius: var(--radius-md);
  font-size: 12px;
  font-weight: 600;
  cursor: pointer;
  transition: all var(--transition-fast);
}

.btn-create:hover {
  background: rgba(20, 184, 166, 0.2);
}

.loading-state, .empty-state {
  text-align: center;
  color: var(--color-text-muted);
  font-size: 13px;
  padding: var(--space-4);
}

.template-form {
  display: flex;
  flex-direction: column;
  gap: var(--space-3);
}

.template-name-input {
  width: 100%;
  background: var(--color-surface-input);
  border: 1px solid var(--color-border-default);
  border-radius: var(--radius-md);
  padding: var(--space-2) var(--space-3);
  color: var(--color-text-primary);
  font-size: 14px;
  font-weight: 600;
}

.template-name-input:focus {
  border-color: var(--color-border-focus);
  outline: none;
}

.template-items-list {
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
}

.template-item-row {
  display: flex;
  align-items: flex-start;
  gap: var(--space-2);
}

.grip-icon {
  color: var(--color-text-muted);
  margin-top: 6px;
  flex-shrink: 0;
}

.template-item-fields {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: var(--space-1);
}

.item-text-input, .item-desc-input {
  width: 100%;
  background: var(--color-surface-input);
  border: 1px solid var(--color-border-default);
  border-radius: var(--radius-sm);
  padding: var(--space-1) var(--space-2);
  color: var(--color-text-primary);
  font-size: 13px;
}

.item-desc-input {
  font-size: 12px;
  color: var(--color-text-secondary);
}

.item-text-input:focus, .item-desc-input:focus {
  border-color: var(--color-border-focus);
  outline: none;
}

.btn-remove-item {
  background: none;
  border: none;
  color: var(--color-text-muted);
  cursor: pointer;
  padding: var(--space-1);
  margin-top: 4px;
}

.btn-remove-item:hover {
  color: var(--color-status-danger);
}

.btn-add-item {
  display: inline-flex;
  align-items: center;
  gap: var(--space-1);
  padding: var(--space-1) var(--space-2);
  background: none;
  border: 1px dashed var(--color-border-default);
  border-radius: var(--radius-md);
  font-size: 12px;
  color: var(--color-text-muted);
  cursor: pointer;
  align-self: flex-start;
}

.btn-add-item:hover {
  border-color: var(--color-accent);
  color: var(--color-accent);
}

.form-actions {
  display: flex;
  gap: var(--space-2);
}

.btn-save {
  padding: var(--space-2) var(--space-4);
  background: var(--color-accent);
  color: white;
  border: none;
  border-radius: var(--radius-md);
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
}

.btn-cancel {
  padding: var(--space-2) var(--space-4);
  background: var(--color-surface-raised);
  color: var(--color-text-secondary);
  border: 1px solid var(--color-border-default);
  border-radius: var(--radius-md);
  font-size: 13px;
  cursor: pointer;
}

.templates-list {
  display: flex;
  flex-direction: column;
  gap: var(--space-3);
}

.template-card {
  background: var(--color-surface-raised);
  border: 1px solid var(--color-border-default);
  border-radius: var(--radius-md);
  padding: var(--space-3);
}

.template-card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: var(--space-2);
}

.template-card-name {
  font-size: 13px;
  font-weight: 600;
  color: var(--color-text-primary);
}

.template-card-count {
  font-size: 11px;
  color: var(--color-text-muted);
}

.template-card-items {
  list-style: none;
  padding: 0;
  margin: 0 0 var(--space-2) 0;
}

.template-card-item {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  font-size: 12px;
  color: var(--color-text-secondary);
  padding: var(--space-1) 0;
}

.item-checkbox-preview {
  color: var(--color-text-muted);
  font-size: 14px;
}

.template-card-actions {
  display: flex;
  gap: var(--space-2);
}

.btn-edit, .btn-delete {
  padding: var(--space-1) var(--space-2);
  background: none;
  border: 1px solid var(--color-border-default);
  border-radius: var(--radius-sm);
  font-size: 11px;
  cursor: pointer;
  color: var(--color-text-muted);
}

.btn-edit:hover {
  color: var(--color-accent);
  border-color: var(--color-accent);
}

.btn-delete:hover {
  color: var(--color-status-danger);
  border-color: var(--color-status-danger);
}
</style>
