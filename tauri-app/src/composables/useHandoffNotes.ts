import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { HandoffNote } from '../types'

/**
 * Composable for managing review handoff notes.
 * Supports creating, listing, deleting, and exporting handoff notes to GitHub.
 */
export function useHandoffNotes() {
  const handoffs = ref<HandoffNote[]>([])
  const loading = ref(false)
  const error = ref<string | null>(null)
  const exporting = ref(false)

  async function fetchHandoffs(prId: number) {
    loading.value = true
    error.value = null
    try {
      handoffs.value = await invoke<HandoffNote[]>('list_handoffs', { prId })
    } catch (e) {
      error.value = String(e)
    } finally {
      loading.value = false
    }
  }

  async function createHandoff(
    prId: number,
    reviewerName: string,
    filesChecked: string[],
    concerns: string,
    remainingWork: string,
  ): Promise<HandoffNote | null> {
    error.value = null
    try {
      const note = await invoke<HandoffNote>('create_handoff', {
        prId,
        reviewerName,
        filesChecked,
        concerns,
        remainingWork,
      })
      handoffs.value.unshift(note)
      return note
    } catch (e) {
      error.value = String(e)
      return null
    }
  }

  async function deleteHandoff(id: number): Promise<boolean> {
    error.value = null
    try {
      await invoke('delete_handoff', { id })
      handoffs.value = handoffs.value.filter(h => h.id !== id)
      return true
    } catch (e) {
      error.value = String(e)
      return false
    }
  }

  async function exportToGitHub(id: number): Promise<string | null> {
    error.value = null
    exporting.value = true
    try {
      const markdown = await invoke<string>('export_handoff_to_github', { id })
      return markdown
    } catch (e) {
      error.value = String(e)
      return null
    } finally {
      exporting.value = false
    }
  }

  return {
    handoffs,
    loading,
    error,
    exporting,
    fetchHandoffs,
    createHandoff,
    deleteHandoff,
    exportToGitHub,
  }
}
