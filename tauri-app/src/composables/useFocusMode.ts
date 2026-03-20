import { ref, onMounted, onUnmounted, watch } from 'vue'

/**
 * Contextual Focus Mode composable.
 *
 * Toggles a `.focus-active` class on `.app-layout` which hides
 * the sidebar and header, giving the main content near-full width.
 * Ambient blobs shift to a calmer state while focus is active.
 *
 * Activated via toggle function or Cmd+Shift+F keyboard shortcut.
 */

const focusActive = ref(false)

/* Apply / remove the class on the layout element */
function applyClass(active: boolean) {
  const layout = document.querySelector('.app-layout')
  if (layout) {
    layout.classList.toggle('focus-active', active)
  }
  /* Also set a data attribute on the app root for blob calming */
  const root = document.querySelector('.app-root')
  if (root) {
    root.classList.toggle('focus-active', active)
  }
}

/* Module-level watcher — runs once when the module is first imported */
watch(focusActive, (val) => {
  applyClass(val)
})

export function useFocusMode() {
  function toggle() {
    focusActive.value = !focusActive.value
  }

  function activate() {
    focusActive.value = true
  }

  function deactivate() {
    focusActive.value = false
  }

  /* Keyboard shortcut: Cmd+Shift+F (Mac) / Ctrl+Shift+F (other) */
  function handleKeydown(e: KeyboardEvent) {
    if ((e.metaKey || e.ctrlKey) && e.shiftKey && e.key.toLowerCase() === 'f') {
      e.preventDefault()
      toggle()
    }
  }

  onMounted(() => {
    window.addEventListener('keydown', handleKeydown)
    /* Re-apply in case the composable is mounted while focus is already on */
    if (focusActive.value) {
      applyClass(true)
    }
  })

  onUnmounted(() => {
    window.removeEventListener('keydown', handleKeydown)
  })

  return {
    focusActive,
    toggle,
    activate,
    deactivate,
  }
}
