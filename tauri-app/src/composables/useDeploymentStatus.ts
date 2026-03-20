import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { Deployment } from '../types'

/**
 * Composable for fetching deployment statuses for a pull request's branch.
 *
 * Calls the `get_deployment_status` Tauri command and exposes
 * the resulting deployments, loading state, and any error.
 */
export function useDeploymentStatus() {
  const deployments = ref<Deployment[]>([])
  const loading = ref(false)
  const error = ref<string | null>(null)

  async function fetchDeployments(prId: number) {
    loading.value = true
    error.value = null
    try {
      deployments.value = await invoke<Deployment[]>('get_deployment_status', { prId })
    } catch (e) {
      error.value = String(e)
      deployments.value = []
    } finally {
      loading.value = false
    }
  }

  return { deployments, loading, error, fetchDeployments }
}
