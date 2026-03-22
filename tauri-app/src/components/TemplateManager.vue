<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { Plus, Trash2, GripVertical } from 'lucide-vue-next'
import { SButton, SInput, STextarea, SCard, SIconButton, SEmptyState } from '@stuntrocket/ui'
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
  <SCard variant="content" class="template-manager">
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
          <SIconButton size="sm" :disabled="index === 0" @click="moveUp(index)" title="Move up">
            <GripVertical :size="14" />
          </SIconButton>
        </div>
        <div class="template-fields">
          <SInput
            v-model="tpl.name"
            placeholder="Template name"
          />
          <STextarea
            v-model="tpl.body"
            placeholder="Review message body"
          />
        </div>
        <div class="template-actions-col">
          <SButton variant="ghost" size="sm" :disabled="index === 0" @click="moveUp(index)" title="Move up">^</SButton>
          <SButton variant="ghost" size="sm" :disabled="index === templates.length - 1" @click="moveDown(index)" title="Move down">v</SButton>
          <SIconButton size="sm" @click="removeTemplate(index)" title="Remove template">
            <Trash2 :size="14" />
          </SIconButton>
        </div>
      </div>

      <SEmptyState
        v-if="templates.length === 0"
        title="No templates"
        description="No review templates configured. Add one to get started."
      />
    </div>

    <div class="manager-footer">
      <SButton variant="secondary" @click="addTemplate">
        <Plus :size="14" />
        Add Template
      </SButton>
      <SButton variant="primary" :disabled="saving" :loading="saving" @click="saveTemplates">
        {{ saved ? 'Saved!' : 'Save Templates' }}
      </SButton>
    </div>
  </SCard>
</template>

<style scoped>
.template-manager {
  overflow: hidden;
}

.manager-header {
  padding-bottom: var(--space-4);
  border-bottom: 1px solid rgba(255, 255, 255, 0.04);
  margin-bottom: var(--space-4);
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
  margin-bottom: var(--space-4);
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

.template-actions-col {
  display: flex;
  flex-direction: column;
  gap: var(--space-1);
  padding-top: var(--space-1);
}

.manager-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding-top: var(--space-4);
  border-top: 1px solid rgba(255, 255, 255, 0.04);
}
</style>
