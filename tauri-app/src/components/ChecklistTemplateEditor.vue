<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { Plus, Trash2, GripVertical } from 'lucide-vue-next'
import { SButton, SInput, SIconButton, SCard, SEmptyState, SSpinner, SSectionHeader } from '@stuntrocket/ui'
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
  <SCard variant="content">
    <div class="editor-header">
      <SSectionHeader title="Review Checklist Templates" />
      <SButton
        v-if="!showCreateForm && !editingTemplate"
        variant="primary"
        size="sm"
        @click="startCreate"
      >
        <Plus :size="14" />
        New Template
      </SButton>
    </div>

    <div v-if="loading" class="loading-state">
      <SSpinner /> Loading templates...
    </div>

    <!-- Create / Edit form -->
    <div v-if="showCreateForm || editingTemplate" class="template-form">
      <SInput
        v-model="editName"
        placeholder="Template name..."
        label="Template Name"
      />
      <div class="template-items-list">
        <div v-for="(item, idx) in editItems" :key="idx" class="template-item-row">
          <GripVertical :size="14" class="grip-icon" />
          <div class="template-item-fields">
            <SInput
              v-model="item.text"
              placeholder="Check item text..."
            />
            <SInput
              v-model="item.description"
              placeholder="Description (optional)..."
            />
          </div>
          <SIconButton size="sm" @click="removeItem(idx)">
            <Trash2 :size="12" />
          </SIconButton>
        </div>
      </div>
      <SButton variant="ghost" size="sm" @click="addItem" class="btn-add-item">
        <Plus :size="12" /> Add Item
      </SButton>
      <div class="form-actions">
        <SButton variant="primary" @click="showCreateForm ? saveCreate() : saveEdit()">
          Save
        </SButton>
        <SButton variant="secondary" @click="cancelEdit">Cancel</SButton>
      </div>
    </div>

    <!-- Existing templates list -->
    <div v-if="!showCreateForm && !editingTemplate" class="templates-list">
      <SCard
        v-for="template in templates"
        :key="template.id"
        variant="nested"
      >
        <div class="template-card-header">
          <span class="template-card-name">{{ template.name }}</span>
          <span class="template-card-count">{{ template.items.length }} items</span>
        </div>
        <ul class="template-card-items">
          <li v-for="item in template.items" :key="item.id" class="template-card-item">
            <span class="item-checkbox-preview">&#9744;</span>
            <span class="item-text">{{ item.text }}</span>
          </li>
        </ul>
        <div class="template-card-actions">
          <SButton variant="ghost" size="sm" @click="startEdit(template)">Edit</SButton>
          <SButton variant="danger" size="sm" @click="handleDelete(template)">Delete</SButton>
        </div>
      </SCard>

      <SEmptyState
        v-if="!loading && templates.length === 0"
        title="No templates"
        description="No checklist templates yet. Create one to standardise reviews for this repository."
      />
    </div>
  </SCard>
</template>

<style scoped>
.editor-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: var(--space-4);
}

.loading-state {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  justify-content: center;
  color: var(--color-text-muted);
  font-size: 13px;
  padding: var(--space-4);
}

.template-form {
  display: flex;
  flex-direction: column;
  gap: var(--space-3);
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

.btn-add-item {
  align-self: flex-start;
}

.form-actions {
  display: flex;
  gap: var(--space-2);
}

.templates-list {
  display: flex;
  flex-direction: column;
  gap: var(--space-3);
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
</style>
