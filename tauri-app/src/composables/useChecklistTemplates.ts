import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { ChecklistTemplate } from '../types'

/**
 * Composable for managing per-repository checklist templates.
 * Templates define ordered lists of review check items that are
 * automatically applied when opening a PR for review.
 */
export function useChecklistTemplates() {
  const templates = ref<ChecklistTemplate[]>([])
  const loading = ref(false)
  const error = ref<string | null>(null)

  async function fetchTemplates(repoId?: number) {
    loading.value = true
    error.value = null
    try {
      const params: Record<string, unknown> = {}
      if (repoId !== undefined) params.repoId = repoId
      templates.value = await invoke<ChecklistTemplate[]>('list_checklist_templates', params)
    } catch (e) {
      error.value = String(e)
    } finally {
      loading.value = false
    }
  }

  async function createTemplate(
    name: string,
    items: Array<{ text: string; description?: string }>,
    repoId?: number,
  ): Promise<ChecklistTemplate | null> {
    error.value = null
    try {
      const itemTuples: [string, string | null][] = items.map(i => [i.text, i.description ?? null])
      const params: Record<string, unknown> = { name, items: itemTuples }
      if (repoId !== undefined) params.repoId = repoId
      const template = await invoke<ChecklistTemplate>('create_checklist_template', params)
      await fetchTemplates(repoId)
      return template
    } catch (e) {
      error.value = String(e)
      return null
    }
  }

  async function updateTemplate(
    templateId: number,
    name: string,
    items: Array<{ text: string; description?: string }>,
    repoId?: number,
  ) {
    error.value = null
    try {
      const itemTuples: [string, string | null][] = items.map(i => [i.text, i.description ?? null])
      await invoke('update_checklist_template', { templateId, name, items: itemTuples })
      await fetchTemplates(repoId)
    } catch (e) {
      error.value = String(e)
    }
  }

  async function deleteTemplate(templateId: number, repoId?: number) {
    error.value = null
    try {
      await invoke('delete_checklist_template', { templateId })
      await fetchTemplates(repoId)
    } catch (e) {
      error.value = String(e)
    }
  }

  /**
   * Get the template items for a specific repository (first matching template).
   */
  function getTemplateForRepo(repoId: number): ChecklistTemplate | undefined {
    return templates.value.find(t => t.repo_id === repoId) ?? templates.value.find(t => t.repo_id === null)
  }

  return {
    templates,
    loading,
    error,
    fetchTemplates,
    createTemplate,
    updateTemplate,
    deleteTemplate,
    getTemplateForRepo,
  }
}
