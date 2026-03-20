import { ref, onUnmounted } from 'vue'

/**
 * Composable for managing hover preview state with a configurable delay.
 * Respects prefers-reduced-motion by disabling the preview entirely.
 */
export function useHoverPreview(delayMs = 400) {
  const hoveredId = ref<number | null>(null)
  const previewPosition = ref({ x: 0, y: 0 })
  const isVisible = ref(false)

  let timerId: ReturnType<typeof setTimeout> | null = null

  /* Respect the user's motion preference — skip hover previews entirely */
  const prefersReducedMotion =
    typeof window !== 'undefined' &&
    window.matchMedia('(prefers-reduced-motion: reduce)').matches

  function onRowEnter(prId: number, event: MouseEvent) {
    if (prefersReducedMotion) return
    clearTimer()
    previewPosition.value = { x: event.clientX, y: event.clientY }
    timerId = setTimeout(() => {
      hoveredId.value = prId
      isVisible.value = true
    }, delayMs)
  }

  function onRowMove(event: MouseEvent) {
    if (!isVisible.value) {
      previewPosition.value = { x: event.clientX, y: event.clientY }
    }
  }

  function onRowLeave() {
    clearTimer()
    hoveredId.value = null
    isVisible.value = false
  }

  function clearTimer() {
    if (timerId !== null) {
      clearTimeout(timerId)
      timerId = null
    }
  }

  onUnmounted(clearTimer)

  return {
    hoveredId,
    previewPosition,
    isVisible,
    onRowEnter,
    onRowMove,
    onRowLeave,
  }
}
