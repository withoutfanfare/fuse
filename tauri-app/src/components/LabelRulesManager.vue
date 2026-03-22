<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { SButton, SInput, SFormField, SSelect, SBadge, SToggle, SIconButton, SCard, SEmptyState, SSpinner } from '@stuntrocket/ui'
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

const actionSelectOptions = actionTypeOptions.map(o => ({ value: o.value, label: o.label }))

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
    <SCard variant="content" class="create-form">
      <div class="form-row">
        <SFormField label="Label Pattern">
          <SInput
            v-model="newPattern"
            placeholder="e.g. bug, priority:*, security*"
          />
        </SFormField>
        <SFormField label="Action">
          <SSelect
            v-model="newActionType"
            :options="actionSelectOptions"
            @update:model-value="updatePlaceholder"
          />
        </SFormField>
        <SFormField label="Configuration" class="form-field-config">
          <SInput
            v-model="newConfigValue"
            :placeholder="currentPlaceholder"
          />
        </SFormField>
        <SButton
          variant="primary"
          :disabled="!newPattern.trim() || creating"
          :loading="creating"
          @click="handleCreate"
        >
          Add Rule
        </SButton>
      </div>
    </SCard>

    <!-- Rules list -->
    <div v-if="loading" class="rules-loading">
      <SSpinner /> Loading rules...
    </div>

    <SEmptyState
      v-else-if="rules.length === 0"
      title="No rules"
      description="No label rules configured yet. Add one above to get started."
    />

    <div v-else class="rules-list">
      <div
        v-for="rule in rules"
        :key="rule.id"
        class="rule-card"
        :class="{ disabled: !rule.enabled }"
      >
        <div class="rule-toggle">
          <SToggle
            :model-value="rule.enabled"
            @update:model-value="handleToggle(rule.id, !rule.enabled)"
          />
        </div>
        <div class="rule-body">
          <div class="rule-pattern">
            <code>{{ rule.label_pattern }}</code>
          </div>
          <div class="rule-action">
            <SBadge variant="accent">{{ actionLabel(rule.action_type) }}</SBadge>
            <span class="action-config">{{ configSummary(rule.action_type, rule.action_config) }}</span>
          </div>
        </div>
        <SIconButton size="sm" @click="handleDelete(rule.id)" title="Delete rule">
          &times;
        </SIconButton>
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
  margin-bottom: var(--space-4);
}

.form-row {
  display: flex;
  gap: var(--space-3);
  align-items: flex-end;
  flex-wrap: wrap;
}

.form-field-config {
  flex: 1;
  min-width: 160px;
}

.rules-loading {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  justify-content: center;
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

.action-config {
  color: var(--color-text-muted);
}
</style>
