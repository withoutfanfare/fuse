<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { SButton, SInput, SCard, SEmptyState } from '@stuntrocket/ui'
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
  <SCard variant="content">
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
        <SInput
          v-model="rules[index]"
          placeholder="Enter review rule..."
        />
        <SButton
          variant="danger"
          size="sm"
          title="Remove rule"
          @click="removeRule(index)"
        >
          &times;
        </SButton>
      </div>
    </div>

    <SEmptyState
      v-if="rules.length === 0"
      title="No review rules defined yet"
      description="Add one below."
    />

    <div class="rules-actions">
      <SButton variant="secondary" size="sm" @click="addRule">
        + Add Rule
      </SButton>
      <SButton
        variant="primary"
        size="sm"
        :disabled="saving"
        :loading="saving"
        @click="saveRules"
      >
        {{ saving ? 'Saving...' : 'Save Rules' }}
      </SButton>
    </div>
  </SCard>
</template>

<style scoped>
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

.rules-actions {
  display: flex;
  gap: var(--space-3);
  align-items: center;
  margin-top: var(--space-4);
}
</style>
