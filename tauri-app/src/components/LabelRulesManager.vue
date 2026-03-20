<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useLabelRules } from '../composables/useLabelRules'
import type { LabelRuleActionType } from '../types'

const { rules, loading, error, fetchRules, createRule, deleteRule, toggleRule } = useLabelRules()

// Form state for creating a new rule
const newPattern = ref('')
const newActionType = ref<LabelRuleActionType>('set_priority')
const newConfigValue = ref('')
const creating = ref(false)

const actionTypeOptions: { value: LabelRuleActionType; label: string; placeholder: string }[] = [
  { value: 'set_priority', label: 'Set Priority', placeholder: 'e.g. high, medium, low' },
  { value: 'add_checklist', label: 'Add Checklist Items', placeholder: 'e.g. Check migrations, Run tests' },
  { value: 'assign_group', label: 'Assign Review Group', placeholder: 'e.g. security-team' },
]

onMounted(() => {
  fetchRules()
})

async function handleCreate() {
  if (!newPattern.value.trim()) return
  creating.value = true

  // Build action_config from the free-text value
  let actionConfig: Record<string, unknown> = {}
  const val = newConfigValue.value.trim()

  if (newActionType.value === 'set_priority') {
    actionConfig = { priority: val || 'high' }
  } else if (newActionType.value === 'add_checklist') {
    // Split by comma for multiple items
    actionConfig = { items: val ? val.split(',').map(s => s.trim()).filter(Boolean) : [] }
  } else if (newActionType.value === 'assign_group') {
    actionConfig = { group: val || '' }
  }

  const result = await createRule(newPattern.value.trim(), newActionType.value, actionConfig)
  if (result) {
    newPattern.value = ''
    newConfigValue.value = ''
  }
  creating.value = false
}

async function handleDelete(id: number) {
  await deleteRule(id)
}

async function handleToggle(id: number, enabled: boolean) {
  await toggleRule(id, enabled)
}

function actionLabel(type: string): string {
  const opt = actionTypeOptions.find(o => o.value === type)
  return opt?.label ?? type
}

function configSummary(type: string, config: Record<string, unknown>): string {
  if (type === 'set_priority') return String(config.priority || '')
  if (type === 'add_checklist') {
    const items = config.items as string[] | undefined
    return items ? items.join(', ') : ''
  }
  if (type === 'assign_group') return String(config.group || '')
  return JSON.stringify(config)
}

const currentPlaceholder = ref('e.g. high, medium, low')
function updatePlaceholder() {
  const opt = actionTypeOptions.find(o => o.value === newActionType.value)
  currentPlaceholder.value = opt?.placeholder ?? ''
}
</script>

<template>
  <div class="label-rules-manager">
    <h3 class="rules-title">Label Automation Rules</h3>
    <p class="rules-description">
      Define rules that trigger actions when a PR carries a matching label.
      Use <code>*</code> as a wildcard (e.g. <code>priority:*</code>).
    </p>

    <p v-if="error" class="rules-error">{{ error }}</p>

    <!-- Create form -->
    <div class="create-form">
      <div class="form-row">
        <div class="form-field">
          <label class="form-label">Label Pattern</label>
          <input
            v-model="newPattern"
            type="text"
            placeholder="e.g. bug, priority:*, security*"
            class="input-field"
          />
        </div>
        <div class="form-field">
          <label class="form-label">Action</label>
          <select v-model="newActionType" class="input-select" @change="updatePlaceholder">
            <option v-for="opt in actionTypeOptions" :key="opt.value" :value="opt.value">
              {{ opt.label }}
            </option>
          </select>
        </div>
        <div class="form-field form-field-config">
          <label class="form-label">Configuration</label>
          <input
            v-model="newConfigValue"
            type="text"
            :placeholder="currentPlaceholder"
            class="input-field"
          />
        </div>
        <button
          class="btn-add"
          :disabled="!newPattern.trim() || creating"
          @click="handleCreate"
        >
          {{ creating ? 'Adding...' : 'Add Rule' }}
        </button>
      </div>
    </div>

    <!-- Rules list -->
    <div v-if="loading" class="rules-loading">Loading rules...</div>

    <div v-else-if="rules.length === 0" class="rules-empty">
      No label rules configured yet. Add one above to get started.
    </div>

    <div v-else class="rules-list">
      <div
        v-for="rule in rules"
        :key="rule.id"
        class="rule-card"
        :class="{ disabled: !rule.enabled }"
      >
        <div class="rule-toggle">
          <input
            type="checkbox"
            :checked="rule.enabled"
            @change="handleToggle(rule.id, !rule.enabled)"
            class="rule-checkbox"
          />
        </div>
        <div class="rule-body">
          <div class="rule-pattern">
            <code>{{ rule.label_pattern }}</code>
          </div>
          <div class="rule-action">
            <span class="action-badge">{{ actionLabel(rule.action_type) }}</span>
            <span class="action-config">{{ configSummary(rule.action_type, rule.action_config) }}</span>
          </div>
        </div>
        <button class="btn-delete" @click="handleDelete(rule.id)" title="Delete rule">
          &times;
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.label-rules-manager {
  margin-bottom: var(--space-8);
}

.rules-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--color-text-primary);
  margin-bottom: var(--space-2);
}

.rules-description {
  font-size: 13px;
  color: var(--color-text-muted);
  margin-bottom: var(--space-4);
}

.rules-description code {
  font-size: 12px;
  font-family: var(--font-mono);
  background: var(--color-surface-raised);
  padding: 1px var(--space-1);
  border-radius: var(--radius-sm);
  color: var(--color-text-secondary);
}

.rules-error {
  color: var(--color-status-danger);
  font-size: 13px;
  margin-bottom: var(--space-3);
}

.create-form {
  background: var(--color-surface-panel);
  border: 1px solid var(--color-border-default);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-card);
  padding: var(--space-4);
  margin-bottom: var(--space-4);
}

.form-row {
  display: flex;
  gap: var(--space-3);
  align-items: flex-end;
  flex-wrap: wrap;
}

.form-field {
  display: flex;
  flex-direction: column;
  gap: var(--space-1);
}

.form-field-config {
  flex: 1;
  min-width: 160px;
}

.form-label {
  font-size: 11px;
  font-weight: 600;
  color: var(--color-text-muted);
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.input-field {
  background: var(--color-surface-input);
  border: 1px solid var(--color-border-default);
  border-radius: var(--radius-md);
  padding: var(--space-2) var(--space-3);
  color: var(--color-text-primary);
  font-size: 13px;
  transition: border-color var(--transition-fast);
  min-width: 140px;
}

.input-field:focus {
  border-color: var(--color-border-focus);
  box-shadow: 0 0 0 2px var(--color-accent-muted);
  outline: none;
}

.input-select {
  background: var(--color-surface-input);
  border: 1px solid var(--color-border-default);
  border-radius: var(--radius-md);
  padding: var(--space-2) var(--space-3);
  color: var(--color-text-primary);
  font-size: 13px;
  transition: border-color var(--transition-fast);
}

.input-select:focus {
  border-color: var(--color-border-focus);
  box-shadow: 0 0 0 2px var(--color-accent-muted);
  outline: none;
}

.btn-add {
  background: var(--color-accent);
  color: var(--color-text-inverse);
  font-weight: 600;
  font-size: 13px;
  padding: var(--space-2) var(--space-4);
  border-radius: var(--radius-md);
  border: none;
  cursor: pointer;
  white-space: nowrap;
  transition: background var(--transition-fast);
}

.btn-add:hover:not(:disabled) {
  background: var(--color-accent-hover);
}

.btn-add:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.rules-loading,
.rules-empty {
  text-align: center;
  padding: var(--space-5);
  color: var(--color-text-muted);
  font-size: 13px;
}

.rules-list {
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
}

.rule-card {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  background: var(--color-surface-panel);
  border: 1px solid var(--color-border-default);
  border-radius: var(--radius-md);
  padding: var(--space-3) var(--space-4);
  transition: all var(--transition-fast);
}

.rule-card.disabled {
  opacity: 0.5;
}

.rule-toggle {
  flex-shrink: 0;
}

.rule-checkbox {
  cursor: pointer;
  accent-color: var(--color-accent);
}

.rule-body {
  flex: 1;
  min-width: 0;
}

.rule-pattern {
  margin-bottom: var(--space-1);
}

.rule-pattern code {
  font-family: var(--font-mono);
  font-size: 13px;
  font-weight: 600;
  color: var(--color-text-primary);
  background: var(--color-surface-raised);
  padding: 1px var(--space-2);
  border-radius: var(--radius-sm);
}

.rule-action {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  font-size: 12px;
}

.action-badge {
  font-weight: 600;
  font-size: 11px;
  padding: 1px var(--space-2);
  border-radius: var(--radius-full);
  background: var(--color-accent-muted);
  color: var(--color-accent);
}

.action-config {
  color: var(--color-text-muted);
}

.btn-delete {
  background: none;
  border: none;
  color: var(--color-text-muted);
  font-size: 18px;
  line-height: 1;
  padding: var(--space-1);
  border-radius: var(--radius-sm);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.btn-delete:hover {
  color: var(--color-status-danger);
  background: rgba(220, 38, 38, 0.1);
}
</style>
