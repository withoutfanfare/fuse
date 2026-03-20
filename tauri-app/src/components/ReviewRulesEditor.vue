<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { usePullRequestsStore } from '../stores/pullRequests'
import { useToastStore } from '../stores/toast'

const props = defineProps<{
  repoId: number
}>()

const prStore = usePullRequestsStore()
const toastStore = useToastStore()

const rules = ref<string[]>([])
const saving = ref(false)
const dragIndex = ref<number | null>(null)
const dragOverIndex = ref<number | null>(null)

onMounted(async () => {
  const existing = await prStore.fetchRules(props.repoId)
  rules.value = existing.map(r => r.rule_text)
})

function addRule() {
  rules.value.push('')
}

function removeRule(index: number) {
  rules.value.splice(index, 1)
}

async function saveRules() {
  const filtered = rules.value.filter(r => r.trim() !== '')
  saving.value = true
  try {
    await prStore.setRules(props.repoId, filtered)
    rules.value = filtered
    toastStore.addToast('success', 'Rules saved', 'Review rules updated successfully')
  } catch {
    toastStore.addToast('error', 'Save failed', 'Could not save review rules')
  } finally {
    saving.value = false
  }
}

/* HTML5 native drag-and-drop reordering */
function onDragStart(index: number, event: DragEvent) {
  dragIndex.value = index
  if (event.dataTransfer) {
    event.dataTransfer.effectAllowed = 'move'
    event.dataTransfer.setData('text/plain', String(index))
  }
}

function onDragOver(index: number, event: DragEvent) {
  event.preventDefault()
  if (event.dataTransfer) {
    event.dataTransfer.dropEffect = 'move'
  }
  dragOverIndex.value = index
}

function onDragLeave() {
  dragOverIndex.value = null
}

function onDrop(index: number, event: DragEvent) {
  event.preventDefault()
  if (dragIndex.value === null || dragIndex.value === index) {
    dragIndex.value = null
    dragOverIndex.value = null
    return
  }

  const moved = rules.value.splice(dragIndex.value, 1)[0]
  rules.value.splice(index, 0, moved)
  dragIndex.value = null
  dragOverIndex.value = null
}

function onDragEnd() {
  dragIndex.value = null
  dragOverIndex.value = null
}
</script>

<template>
  <div class="rules-editor">
    <h4 class="rules-title">Review Rules</h4>
    <p class="rules-description">
      Define rules that reviewers should check for every PR in this repository.
    </p>

    <div class="rules-list">
      <div
        v-for="(_rule, index) in rules"
        :key="index"
        class="rule-row"
        :class="{
          dragging: dragIndex === index,
          'drag-over': dragOverIndex === index && dragIndex !== index,
        }"
        draggable="true"
        @dragstart="onDragStart(index, $event)"
        @dragover="onDragOver(index, $event)"
        @dragleave="onDragLeave"
        @drop="onDrop(index, $event)"
        @dragend="onDragEnd"
      >
        <span class="drag-handle" title="Drag to reorder">&#x2630;</span>
        <input
          v-model="rules[index]"
          type="text"
          class="rule-input"
          placeholder="Enter review rule…"
        />
        <button
          class="btn-delete-rule"
          title="Remove rule"
          @click="removeRule(index)"
        >
          &times;
        </button>
      </div>
    </div>

    <div v-if="rules.length === 0" class="rules-empty">
      No review rules defined yet. Add one below.
    </div>

    <div class="rules-actions">
      <button class="btn-add-rule" @click="addRule">
        + Add Rule
      </button>
      <button
        class="btn-save-rules"
        :disabled="saving"
        @click="saveRules"
      >
        {{ saving ? 'Saving…' : 'Save Rules' }}
      </button>
    </div>
  </div>
</template>

<style scoped>
.rules-editor {
  background: var(--color-surface-panel);
  border: 1px solid var(--color-border-default);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-card);
  padding: var(--space-5);
  margin-top: var(--space-3);
}

.rules-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--color-text-primary);
  margin-bottom: var(--space-1);
}

.rules-description {
  font-size: 12px;
  color: var(--color-text-muted);
  margin-bottom: var(--space-4);
}

.rules-list {
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
  margin-bottom: var(--space-4);
}

.rule-row {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  padding: var(--space-2);
  border-radius: var(--radius-md);
  border: 1px solid transparent;
  background: var(--color-surface-raised);
  transition: background var(--transition-fast), border-color var(--transition-fast),
    opacity var(--transition-fast), box-shadow var(--transition-fast);
}

.rule-row.dragging {
  opacity: 0.4;
}

.rule-row.drag-over {
  border-color: var(--color-accent);
  background: var(--color-accent-muted);
  box-shadow: 0 0 0 1px var(--color-accent);
}

.drag-handle {
  cursor: grab;
  color: var(--color-text-muted);
  font-size: 14px;
  padding: 0 var(--space-1);
  user-select: none;
  flex-shrink: 0;
}

.drag-handle:active {
  cursor: grabbing;
}

.rule-input {
  flex: 1;
  background: var(--color-surface-input);
  border: 1px solid var(--color-border-default);
  border-radius: var(--radius-sm);
  padding: var(--space-2) var(--space-3);
  color: var(--color-text-primary);
  font-size: 13px;
  transition: border-color var(--transition-fast);
}

.rule-input:focus {
  border-color: var(--color-border-focus);
  box-shadow: 0 0 0 2px var(--color-accent-muted);
  outline: none;
}

.rule-input::placeholder {
  color: var(--color-text-muted);
}

.btn-delete-rule {
  width: 28px;
  height: 28px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  color: var(--color-text-muted);
  border: 1px solid transparent;
  border-radius: var(--radius-sm);
  cursor: pointer;
  font-size: 16px;
  flex-shrink: 0;
  transition: all var(--transition-fast);
}

.btn-delete-rule:hover {
  color: var(--color-status-danger);
  background: rgba(220, 38, 38, 0.1);
  border-color: rgba(220, 38, 38, 0.2);
}

.btn-delete-rule:active {
  transform: scale(0.95);
}

.rules-empty {
  font-size: 13px;
  color: var(--color-text-muted);
  text-align: center;
  padding: var(--space-4);
  background: var(--color-surface-raised);
  border-radius: var(--radius-md);
  margin-bottom: var(--space-4);
}

.rules-actions {
  display: flex;
  gap: var(--space-3);
  align-items: center;
}

.btn-add-rule {
  background: var(--color-surface-raised);
  color: var(--color-text-secondary);
  font-size: 13px;
  font-weight: 500;
  padding: var(--space-2) var(--space-4);
  border-radius: var(--radius-md);
  border: 1px solid var(--color-border-default);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.btn-add-rule:hover {
  background: var(--color-surface-hover);
  border-color: var(--color-border-hover);
  color: var(--color-text-primary);
}

.btn-add-rule:active {
  transform: scale(0.97);
}

.btn-add-rule:focus-visible {
  box-shadow: 0 0 0 2px var(--color-accent-muted);
  outline: none;
}

.btn-save-rules {
  background: var(--color-accent);
  color: var(--color-text-inverse);
  font-size: 13px;
  font-weight: 600;
  padding: var(--space-2) var(--space-4);
  border-radius: var(--radius-md);
  border: none;
  cursor: pointer;
  transition: background var(--transition-fast), transform var(--transition-fast);
}

.btn-save-rules:hover:not(:disabled) {
  background: var(--color-accent-hover);
}

.btn-save-rules:active:not(:disabled) {
  transform: scale(0.97);
}

.btn-save-rules:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn-save-rules:focus-visible {
  box-shadow: 0 0 0 2px var(--color-accent-muted);
  outline: none;
}
</style>
