import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { LabelRule, LabelRuleActionType, LabelRuleMatch } from '../types'

/**
 * Composable for managing label-based automation rules.
 * Provides CRUD operations and rule evaluation against PR labels.
 */
export function useLabelRules() {
  const rules = ref<LabelRule[]>([])
  const loading = ref(false)
  const error = ref<string | null>(null)

  async function fetchRules() {
    loading.value = true
    error.value = null
    try {
      rules.value = await invoke<LabelRule[]>('list_label_rules')
    } catch (e) {
      error.value = String(e)
    } finally {
      loading.value = false
    }
  }

  async function createRule(
    labelPattern: string,
    actionType: LabelRuleActionType,
    actionConfig: Record<string, unknown>,
  ): Promise<LabelRule | null> {
    error.value = null
    try {
      const rule = await invoke<LabelRule>('create_label_rule', {
        labelPattern,
        actionType,
        actionConfig,
      })
      rules.value.push(rule)
      return rule
    } catch (e) {
      error.value = String(e)
      return null
    }
  }

  async function deleteRule(id: number): Promise<boolean> {
    error.value = null
    try {
      await invoke('delete_label_rule', { id })
      rules.value = rules.value.filter(r => r.id !== id)
      return true
    } catch (e) {
      error.value = String(e)
      return false
    }
  }

  async function toggleRule(id: number, enabled: boolean): Promise<boolean> {
    error.value = null
    try {
      await invoke('toggle_label_rule', { id, enabled })
      const rule = rules.value.find(r => r.id === id)
      if (rule) rule.enabled = enabled
      return true
    } catch (e) {
      error.value = String(e)
      return false
    }
  }

  async function evaluateRules(prId: number): Promise<LabelRuleMatch[]> {
    error.value = null
    try {
      return await invoke<LabelRuleMatch[]>('evaluate_label_rules', { prId })
    } catch (e) {
      error.value = String(e)
      return []
    }
  }

  return { rules, loading, error, fetchRules, createRule, deleteRule, toggleRule, evaluateRules }
}
