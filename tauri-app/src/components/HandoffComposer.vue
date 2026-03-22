<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { SButton, SIconButton, SInput, STextarea, SCard, SCheckbox, SBadge, SSpinner, SEmptyState } from '@stuntrocket/ui'
import { useHandoffNotes } from '../composables/useHandoffNotes'
import { useToastStore } from '../stores/toast'

const props = defineProps<{
  prId: number
  /** File paths available in this PR (from diff) for pre-populating the file checklist. */
  availableFiles?: string[]
}>()

const toastStore = useToastStore()
const {
  handoffs,
  loading,
  error,
  exporting,
  fetchHandoffs,
  createHandoff,
  deleteHandoff,
  exportToGitHub,
} = useHandoffNotes()

const expanded = ref(false)
const showForm = ref(false)
const reviewerName = ref('')
const selectedFiles = ref<string[]>([])
const concerns = ref('')
const remainingWork = ref('')

onMounted(() => {
  fetchHandoffs(props.prId)
})

function toggleFile(filePath: string) {
  const idx = selectedFiles.value.indexOf(filePath)
  if (idx === -1) {
    selectedFiles.value.push(filePath)
  } else {
    selectedFiles.value.splice(idx, 1)
  }
}

async function handleCreate() {
  if (!reviewerName.value.trim()) return
  const note = await createHandoff(
    props.prId,
    reviewerName.value.trim(),
    selectedFiles.value,
    concerns.value.trim(),
    remainingWork.value.trim(),
  )
  if (note) {
    toastStore.addToast('success', 'Handoff Created', 'Review handoff note saved.')
    resetForm()
  }
}

async function handleExport(id: number) {
  const markdown = await exportToGitHub(id)
  if (markdown) {
    toastStore.addToast('success', 'Exported', 'Handoff note posted as GitHub comment.')
  } else if (error.value) {
    toastStore.addToast('error', 'Export Failed', error.value)
  }
}

async function handleDelete(id: number) {
  await deleteHandoff(id)
}

function resetForm() {
  showForm.value = false
  reviewerName.value = ''
  selectedFiles.value = []
  concerns.value = ''
  remainingWork.value = ''
}

function formatDate(dateStr: string): string {
  return new Date(dateStr).toLocaleDateString('en-GB', {
    day: 'numeric', month: 'short', year: 'numeric',
    hour: '2-digit', minute: '2-digit',
  })
}
</script>

<template>
  <section class="handoff-panel">
    <div class="handoff-toggle" role="button" tabindex="0" @click="expanded = !expanded" @keydown.enter="expanded = !expanded" @keydown.space.prevent="expanded = !expanded">
      <h2 class="section-title">Review Handoffs</h2>
      <SBadge v-if="handoffs.length > 0" variant="warning">{{ handoffs.length }}</SBadge>
      <SIconButton
        variant="ghost"
        size="sm"
        :tooltip="expanded ? 'Collapse' : 'Expand'"
        @click.stop="expanded = !expanded"
      >
        <span class="toggle-icon" :class="{ expanded }">&#9656;</span>
      </SIconButton>
    </div>

    <div v-if="expanded" class="handoff-content">
      <div v-if="loading" class="handoff-loading">
        <SSpinner /> Loading handoff notes...
      </div>

      <div v-else>
        <!-- Existing handoff notes -->
        <div v-if="handoffs.length > 0" class="handoff-list">
          <SCard
            v-for="note in handoffs"
            :key="note.id"
            variant="nested"
          >
            <div class="handoff-card-header">
              <span class="handoff-reviewer">{{ note.reviewer_name }}</span>
              <span class="handoff-date">{{ formatDate(note.created_at) }}</span>
            </div>

            <div v-if="note.files_checked.length > 0" class="handoff-files">
              <span class="handoff-label">Files reviewed:</span>
              <div class="handoff-file-list">
                <code
                  v-for="file in note.files_checked"
                  :key="file"
                  class="handoff-file"
                >
                  {{ file }}
                </code>
              </div>
            </div>

            <div v-if="note.concerns" class="handoff-section">
              <span class="handoff-label">Concerns:</span>
              <p class="handoff-text">{{ note.concerns }}</p>
            </div>

            <div v-if="note.remaining_work" class="handoff-section">
              <span class="handoff-label">Remaining work:</span>
              <p class="handoff-text">{{ note.remaining_work }}</p>
            </div>

            <div class="handoff-card-actions">
              <SButton
                variant="secondary"
                size="sm"
                :disabled="exporting"
                :loading="exporting"
                @click="handleExport(note.id)"
              >
                Post to GitHub
              </SButton>
              <SButton
                variant="danger"
                size="sm"
                @click="handleDelete(note.id)"
              >
                Delete
              </SButton>
            </div>
          </SCard>
        </div>

        <SEmptyState
          v-else-if="!showForm"
          title="No handoff notes"
          description="No handoff notes yet. Create one to hand off this review."
        />

        <!-- Create form -->
        <SCard v-if="showForm" variant="nested" class="handoff-form">
          <div class="form-group">
            <SInput
              v-model="reviewerName"
              label="Your name"
              placeholder="e.g. Jane Smith"
            />
          </div>

          <div v-if="availableFiles && availableFiles.length > 0" class="form-group">
            <label class="form-label">Files reviewed</label>
            <div class="file-checklist">
              <label
                v-for="file in availableFiles"
                :key="file"
                class="file-check-item"
                :class="{ checked: selectedFiles.includes(file) }"
              >
                <SCheckbox
                  :model-value="selectedFiles.includes(file)"
                  @update:model-value="toggleFile(file)"
                />
                <code class="file-check-path">{{ file }}</code>
              </label>
            </div>
          </div>

          <div class="form-group">
            <STextarea
              v-model="concerns"
              label="Concerns"
              placeholder="Any concerns or issues found during review..."
            />
          </div>

          <div class="form-group">
            <STextarea
              v-model="remainingWork"
              label="Remaining work"
              placeholder="What still needs to be reviewed or checked..."
            />
          </div>

          <div class="form-actions">
            <SButton variant="primary" @click="handleCreate">
              Create Handoff
            </SButton>
            <SButton variant="secondary" @click="resetForm">
              Cancel
            </SButton>
          </div>
        </SCard>

        <SButton
          v-if="!showForm"
          variant="ghost"
          class="btn-new-handoff"
          @click="showForm = true"
        >
          + New Handoff Note
        </SButton>
      </div>

      <div v-if="error" class="handoff-error">{{ error }}</div>
    </div>
  </section>
</template>

<style scoped>
.handoff-panel {
  padding: 0;
}

.handoff-toggle {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  width: 100%;
  padding: var(--space-5);
  cursor: pointer;
  color: inherit;
  text-align: left;
  transition: background var(--transition-fast);
  border-radius: var(--radius-lg);
}

.handoff-toggle:hover {
  background: rgba(255, 255, 255, 0.02);
}

.section-title {
  font-size: 16px;
  font-weight: 600;
  color: var(--color-text-primary);
  margin-bottom: var(--space-4);
}

.handoff-toggle .section-title {
  margin-bottom: 0;
  flex: 1;
}

.toggle-icon {
  font-size: 14px;
  color: var(--color-text-muted);
  transition: transform var(--transition-fast);
  display: inline-block;
}

.toggle-icon.expanded {
  transform: rotate(90deg);
}

.handoff-content {
  padding: 0 var(--space-5) var(--space-5);
  border-top: 1px solid var(--color-border-default);
  padding-top: var(--space-4);
}

.handoff-loading {
  text-align: center;
  color: var(--color-text-muted);
  font-size: 13px;
  padding: var(--space-2);
  display: flex;
  align-items: center;
  justify-content: center;
  gap: var(--space-2);
}

.handoff-list {
  display: flex;
  flex-direction: column;
  gap: var(--space-3);
}

.handoff-card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.handoff-reviewer {
  font-weight: 600;
  font-size: 14px;
  color: var(--color-text-primary);
}

.handoff-date {
  font-size: 12px;
  color: var(--color-text-muted);
  font-family: var(--font-mono);
}

.handoff-label {
  font-size: 12px;
  font-weight: 600;
  color: var(--color-text-muted);
  text-transform: uppercase;
  letter-spacing: 0.03em;
}

.handoff-files {
  display: flex;
  flex-direction: column;
  gap: var(--space-1);
}

.handoff-file-list {
  display: flex;
  flex-wrap: wrap;
  gap: var(--space-1);
}

.handoff-file {
  font-size: 11px;
  font-family: var(--font-mono);
  background: rgba(20, 184, 166, 0.1);
  color: var(--color-accent);
  padding: 1px var(--space-2);
  border-radius: var(--radius-sm);
}

.handoff-section {
  display: flex;
  flex-direction: column;
  gap: var(--space-1);
}

.handoff-text {
  font-size: 13px;
  color: var(--color-text-secondary);
  margin: 0;
  line-height: 1.5;
  white-space: pre-wrap;
}

.handoff-card-actions {
  display: flex;
  gap: var(--space-2);
  padding-top: var(--space-2);
  border-top: 1px solid rgba(255, 255, 255, 0.04);
}

/* Create form */
.handoff-form {
  margin-top: var(--space-3);
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
  margin-bottom: var(--space-3);
}

.form-label {
  font-size: 12px;
  font-weight: 600;
  color: var(--color-text-muted);
}

.file-checklist {
  max-height: 200px;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: var(--space-1);
}

.file-check-item {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  padding: var(--space-1) var(--space-2);
  border-radius: var(--radius-sm);
  cursor: pointer;
  transition: background var(--transition-fast);
}

.file-check-item:hover {
  background: var(--color-surface-hover);
}

.file-check-item.checked .file-check-path {
  color: var(--color-accent);
}

.file-check-path {
  font-size: 12px;
  font-family: var(--font-mono);
  color: var(--color-text-secondary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.form-actions {
  display: flex;
  gap: var(--space-2);
}

.btn-new-handoff {
  width: 100%;
  margin-top: var(--space-3);
}

.handoff-error {
  color: var(--color-status-danger);
  font-size: 12px;
  margin-top: var(--space-2);
}
</style>
