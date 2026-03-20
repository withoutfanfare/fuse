import { ref, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export function useReviewTimer(prId: number) {
  const elapsed = ref(0)
  const running = ref(false)

  let intervalId: ReturnType<typeof setInterval> | null = null
  let lastSavedElapsed = 0

  function start() {
    if (running.value) return
    running.value = true
    intervalId = setInterval(() => {
      elapsed.value++
    }, 1000)
  }

  function pause() {
    if (!running.value) return
    running.value = false
    if (intervalId !== null) {
      clearInterval(intervalId)
      intervalId = null
    }
  }

  function resume() {
    start()
  }

  function stop() {
    pause()
    saveTime()
  }

  async function saveTime() {
    const secondsToSave = elapsed.value - lastSavedElapsed
    if (secondsToSave <= 0) return
    try {
      await invoke('record_review_time', { prId, seconds: secondsToSave })
      lastSavedElapsed = elapsed.value
    } catch {
      // Silently fail — we'll try again on next save
    }
  }

  function handleVisibilityChange() {
    if (document.hidden) {
      pause()
    } else {
      start()
    }
  }

  onMounted(() => {
    document.addEventListener('visibilitychange', handleVisibilityChange)
    start()
  })

  onUnmounted(() => {
    document.removeEventListener('visibilitychange', handleVisibilityChange)
    stop()
  })

  return {
    elapsed,
    start,
    stop,
    pause,
    resume,
  }
}
