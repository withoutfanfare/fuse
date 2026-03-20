import { ref, computed } from 'vue'
import { useRepositoriesStore } from '../stores/repositories'

const DISMISS_KEY = 'pr-review-onboarding-dismissed'

const currentStep = ref(0)

export function useOnboarding() {
  const repoStore = useRepositoriesStore()

  const showOnboarding = computed(() => {
    const dismissed = localStorage.getItem(DISMISS_KEY) === 'true'
    return !dismissed && repoStore.repos.length === 0
  })

  function dismissOnboarding() {
    localStorage.setItem(DISMISS_KEY, 'true')
    currentStep.value = 0
  }

  function completeOnboarding() {
    localStorage.setItem(DISMISS_KEY, 'true')
    currentStep.value = 0
  }

  return {
    showOnboarding,
    currentStep,
    dismissOnboarding,
    completeOnboarding,
  }
}
