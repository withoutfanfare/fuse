<script setup lang="ts">
import { ref, onMounted } from 'vue'
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
    <button class="handoff-toggle" @click="expanded = !expanded">
      <h2 class="section-title">Review Handoffs</h2>
      <span class="handoff-count" v-if="handoffs.length > 0">{{ handoffs.length }}</span>
      <span class="toggle-icon" :class="{ expanded }">&#9656;</span>
    </button>

    <div v-if="expanded" class="handoff-content">
      <div v-if="loading" class="handoff-loading">Loading handoff notes...</div>

      <div v-else>
        <!-- Existing handoff notes -->
        <div v-if="handoffs.length > 0" class="handoff-list">
          <div
            v-for="note in handoffs"
            :key="note.id"
            class="handoff-card"
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
              <button
                class="btn-export"
                :disabled="exporting"
                @click="handleExport(note.id)"
              >
                {{ exporting ? 'Exporting...' : 'Post to GitHub' }}
              </button>
              <button
                class="btn-delete-handoff"
                @click="handleDelete(note.id)"
              >
                Delete
              </button>
            </div>
          </div>
        </div>

        <div v-else-if="!showForm" class="handoff-empty">
          No handoff notes yet. Create one to hand off this review.
        </div>

        <!-- Create form -->
        <div v-if="showForm" class="handoff-form">
          <div class="form-group">
            <label class="form-label">Your name</label>
            <input
              v-model="reviewerName"
              class="form-input"
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
                <input
                  type="checkbox"
                  :checked="selectedFiles.includes(file)"
                  class="file-checkbox"
                  @change="toggleFile(file)"
                />
                <code class="file-check-path">{{ file }}</code>
              </label>
            </div>
          </div>

          <div class="form-group">
            <label class="form-label">Concerns</label>
            <textarea
              v-model="concerns"
              class="form-textarea"
              placeholder="Any concerns or issues found during review..."
              rows="3"
            />
          </div>

          <div class="form-group">
            <label class="form-label">Remaining work</label>
            <textarea
              v-model="remainingWork"
              class="form-textarea"
              placeholder="What still needs to be reviewed or checked..."
              rows="3"
            />
          </div>

          <div class="form-actions">
            <button class="btn-create-handoff" @click="handleCreate">
              Create Handoff
            </button>
            <button class="btn-cancel-handoff" @click="resetForm">
              Cancel
            </button>
          </div>
        </div>

        <button
          v-if="!showForm"
          class="btn-new-handoff"
          @click="showForm = true"
        >
          + New Handoff Note
        </button>
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
  background: none;
  border: none;
  cursor: pointer;
  color: inherit;
  text-align: left;
  transition: background var(--transition-fast);
  border-radius: var(--radius-lg);
}

.handoff-toggle:hover {
  background: rgba(255, 255, 255, 0.02);
}

.handoff-toggle .section-title {
  margin-bottom: 0;
  flex: 1;
}

.handoff-count {
  background: rgba(234, 179, 8, 0.2);
  color: var(--color-status-warning);
  font-size: 11px;
  font-weight: 700;
  min-width: 20px;
  height: 20px;
  border-radius: var(--radius-full);
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 0 var(--space-1);
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

.handoff-loading,
.handoff-empty {
  text-align: center;
  color: var(--color-text-muted);
  font-size: 13px;
  padding: var(--space-2);
}

.handoff-list {
  display: flex;
  flex-direction: column;
  gap: var(--space-3);
}

.handoff-card {
  background: var(--color-surface-raised);
  border: 1px solid var(--color-border-default);
  border-radius: var(--radius-md);
  padding: var(--space-4);
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

.btn-export {
  font-size: 12px;
  padding: var(--space-1) var(--space-3);
  border-radius: var(--radius-md);
  background: rgba(59, 130, 246, 0.2);
  color: var(--color-status-info);
  border: 1px solid rgba(59, 130, 246, 0.3);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.btn-export:hover:not(:disabled) {
  background: rgba(59, 130, 246, 0.3);
}

.btn-export:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn-delete-handoff {
  font-size: 12px;
  padding: var(--space-1) var(--space-3);
  border-radius: var(--radius-md);
  background: rgba(220, 38, 38, 0.1);
  color: var(--color-status-danger);
  border: 1px solid rgba(220, 38, 38, 0.2);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.btn-delete-handoff:hover {
  background: rgba(220, 38, 38, 0.2);
}

/* Create form */
.handoff-form {
  margin-top: var(--space-3);
  padding: var(--space-4);
  background: var(--color-surface-raised);
  border: 1px solid var(--color-border-default);
  border-radius: var(--radius-md);
  display: flex;
  flex-direction: column;
  gap: var(--space-4);
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
}

.form-label {
  font-size: 12px;
  font-weight: 600;
  color: var(--color-text-muted);
}

.form-input {
  width: 100%;
  background: var(--color-surface-input);
  border: 1px solid var(--color-border-default);
  border-radius: var(--radius-md);
  padding: var(--space-2) var(--space-3);
  color: var(--color-text-primary);
  font-size: 13px;
  font-family: var(--font-sans);
  transition: border-color var(--transition-fast);
}

.form-input:focus {
  border-color: var(--color-border-focus);
  box-shadow: 0 0 0 2px var(--color-accent-muted);
  outline: none;
}

.form-textarea {
  width: 100%;
  background: var(--color-surface-input);
  border: 1px solid var(--color-border-default);
  border-radius: var(--radius-md);
  padding: var(--space-2) var(--space-3);
  color: var(--color-text-primary);
  font-size: 13px;
  font-family: var(--font-sans);
  resize: vertical;
  min-height: 60px;
  transition: border-color var(--transition-fast);
}

.form-textarea:focus {
  border-color: var(--color-border-focus);
  box-shadow: 0 0 0 2px var(--color-accent-muted);
  outline: none;
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

.file-checkbox {
  appearance: none;
  width: 14px;
  height: 14px;
  border: 1px solid var(--color-border-default);
  border-radius: var(--radius-sm);
  background: var(--color-surface-input);
  flex-shrink: 0;
  cursor: pointer;
  transition: all var(--transition-fast);
  position: relative;
}

.file-checkbox:checked {
  background: var(--color-accent);
  border-color: var(--color-accent);
}

.file-checkbox:checked::after {
  content: '';
  position: absolute;
  left: 3px;
  top: 0px;
  width: 5px;
  height: 8px;
  border: solid var(--color-text-inverse);
  border-width: 0 2px 2px 0;
  transform: rotate(45deg);
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

.btn-create-handoff {
  font-size: 13px;
  font-weight: 600;
  padding: var(--space-2) var(--space-4);
  border-radius: var(--radius-md);
  background: rgba(234, 179, 8, 0.2);
  color: var(--color-status-warning);
  border: 1px solid rgba(234, 179, 8, 0.3);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.btn-create-handoff:hover {
  background: rgba(234, 179, 8, 0.3);
}

.btn-cancel-handoff {
  font-size: 13px;
  padding: var(--space-2) var(--space-4);
  border-radius: var(--radius-md);
  background: var(--color-surface-raised);
  color: var(--color-text-secondary);
  border: 1px solid var(--color-border-default);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.btn-cancel-handoff:hover {
  background: var(--color-surface-hover);
}

.btn-new-handoff {
  width: 100%;
  margin-top: var(--space-3);
  padding: var(--space-2);
  background: none;
  border: 1px dashed var(--color-border-default);
  border-radius: var(--radius-md);
  color: var(--color-text-muted);
  font-size: 13px;
  cursor: pointer;
  transition: all var(--transition-fast);
}

.btn-new-handoff:hover {
  border-color: rgba(234, 179, 8, 0.5);
  color: var(--color-status-warning);
  background: rgba(234, 179, 8, 0.05);
}

.handoff-error {
  color: var(--color-status-danger);
  font-size: 12px;
  margin-top: var(--space-2);
}
</style>
