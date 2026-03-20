import { ref, watch, onMounted, onUnmounted } from 'vue'
import { useSettingsStore } from '../stores/settings'

/* -------------------------------------------------------
   Theme mode — Dark, Light, or System (auto-detect)
   ------------------------------------------------------- */
export type ThemeMode = 'dark' | 'light' | 'system'

const THEME_KEY = 'fuse-theme-mode'

/* -------------------------------------------------------
   Accent colour presets — each provides HSL components
   so derived tokens (muted, hover) can be computed.
   ------------------------------------------------------- */
export interface AccentPreset {
  name: string
  hue: number
  sat: number
  light: number
}

export const ACCENT_PRESETS: AccentPreset[] = [
  { name: 'Teal',    hue: 168, sat: 80, light: 41 },
  { name: 'Violet',  hue: 258, sat: 90, light: 66 },
  { name: 'Blue',    hue: 217, sat: 91, light: 60 },
  { name: 'Amber',   hue: 38,  sat: 92, light: 50 },
  { name: 'Rose',    hue: 347, sat: 77, light: 50 },
  { name: 'Emerald', hue: 152, sat: 69, light: 41 },
  { name: 'Orange',  hue: 25,  sat: 95, light: 53 },
]

const ACCENT_KEY = 'fuse-accent-colour'

/**
 * Composable managing theme mode (dark / light / system) and
 * accent colour. Applies `data-theme` attribute on `<html>`
 * and accent CSS custom properties on `documentElement.style`.
 *
 * Persists choices via the settings store with a localStorage
 * fallback — same pattern as useDensity.
 */
export function useTheme() {
  const store = useSettingsStore()

  const themeMode = ref<ThemeMode>('dark')
  const accentName = ref<string>('Teal')

  /* System preference media query */
  let mql: MediaQueryList | null = null
  let mqlHandler: ((e: MediaQueryListEvent) => void) | null = null

  /* ---- Theme mode ---- */

  /** Resolve the effective appearance ('dark' | 'light') from the current mode. */
  function resolvedTheme(): 'dark' | 'light' {
    if (themeMode.value === 'system') {
      return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light'
    }
    return themeMode.value
  }

  function applyTheme() {
    const effective = resolvedTheme()
    document.documentElement.setAttribute('data-theme', effective)
  }

  function setThemeMode(mode: ThemeMode) {
    themeMode.value = mode
    applyTheme()
    localStorage.setItem(THEME_KEY, mode)
    store.updateSetting('theme_mode', mode).catch(() => {
      /* Silently fall back to localStorage */
    })
  }

  /* ---- Accent colour ---- */

  function applyAccent(preset: AccentPreset) {
    const { hue, sat, light } = preset
    const style = document.documentElement.style
    style.setProperty('--color-accent', `hsl(${hue}, ${sat}%, ${light}%)`)
    style.setProperty('--color-accent-hover', `hsl(${hue}, ${sat}%, ${Math.min(light + 10, 85)}%)`)
    style.setProperty('--color-accent-muted', `hsla(${hue}, ${sat}%, ${light}%, 0.2)`)
    /* Border focus mirrors the accent */
    style.setProperty('--color-border-focus', `hsl(${hue}, ${sat}%, ${light}%)`)
  }

  function setAccent(name: string) {
    const preset = ACCENT_PRESETS.find(p => p.name === name)
    if (!preset) return
    accentName.value = name
    applyAccent(preset)
    localStorage.setItem(ACCENT_KEY, name)
    store.updateSetting('accent_colour', name).catch(() => {
      /* Silently fall back to localStorage */
    })
  }

  /* ---- Lifecycle ---- */

  onMounted(async () => {
    await store.fetchSettings()

    /* Hydrate theme mode */
    const storedMode =
      (store.settings.theme_mode as ThemeMode) ||
      (localStorage.getItem(THEME_KEY) as ThemeMode) ||
      'dark'
    themeMode.value = storedMode
    applyTheme()

    /* Listen for system preference changes when mode is 'system' */
    mql = window.matchMedia('(prefers-color-scheme: dark)')
    mqlHandler = () => {
      if (themeMode.value === 'system') applyTheme()
    }
    mql.addEventListener('change', mqlHandler)

    /* Hydrate accent colour */
    const storedAccent =
      store.settings.accent_colour ||
      localStorage.getItem(ACCENT_KEY) ||
      'Teal'
    const preset = ACCENT_PRESETS.find(p => p.name === storedAccent)
    if (preset) {
      accentName.value = storedAccent
      applyAccent(preset)
    }
  })

  onUnmounted(() => {
    if (mql && mqlHandler) {
      mql.removeEventListener('change', mqlHandler)
    }
  })

  watch(themeMode, applyTheme)

  return {
    themeMode,
    accentName,
    setThemeMode,
    setAccent,
    ACCENT_PRESETS,
  }
}
