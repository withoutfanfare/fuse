<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useSettingsStore } from '../stores/settings'
import { useDensity } from '../composables/useDensity'
import { useTheme, ACCENT_PRESETS } from '../composables/useTheme'
import TemplateManager from '../components/TemplateManager.vue'
import LabelRulesManager from '../components/LabelRulesManager.vue'

const store = useSettingsStore()
const { density, setDensity } = useDensity()
const { themeMode, accentName, setThemeMode, setAccent } = useTheme()
const pollInterval = ref(300)
const editorCommand = ref('code')
const staleThresholdDays = ref(14)
const ligaturesEnabled = ref(true)
const saved = ref(false)
const saving = ref(false)

const highContrastEnabled = ref(false)
const HIGH_CONTRAST_KEY = 'fuse-high-contrast'

/** Apply or remove the data-high-contrast attribute on <html>. */
function applyHighContrast(enabled: boolean) {
  if (enabled) {
    document.documentElement.setAttribute('data-high-contrast', '')
  } else {
    document.documentElement.removeAttribute('data-high-contrast')
  }
}

function toggleHighContrast() {
  highContrastEnabled.value = !highContrastEnabled.value
  applyHighContrast(highContrastEnabled.value)
  localStorage.setItem(HIGH_CONTRAST_KEY, String(highContrastEnabled.value))
  store.updateSetting('high_contrast', String(highContrastEnabled.value)).catch(() => {
    /* Silently fall back to localStorage */
  })
}

const LIGATURES_KEY = 'fuse-ligatures-enabled'

/** Apply or remove the ligatures-off class on #app */
function applyLigatures(enabled: boolean) {
  const root = document.getElementById('app')
  if (!root) return
  if (enabled) {
    root.classList.remove('ligatures-off')
  } else {
    root.classList.add('ligatures-off')
  }
}

function toggleLigatures() {
  ligaturesEnabled.value = !ligaturesEnabled.value
  applyLigatures(ligaturesEnabled.value)
  localStorage.setItem(LIGATURES_KEY, String(ligaturesEnabled.value))
  store.updateSetting('code_ligatures', String(ligaturesEnabled.value)).catch(() => {
    /* Silently fall back to localStorage */
  })
}

onMounted(async () => {
  await store.fetchSettings()
  if (store.settings.poll_interval_seconds) {
    pollInterval.value = Number(store.settings.poll_interval_seconds)
  }
  if (store.settings.editor_command) {
    editorCommand.value = store.settings.editor_command
  }
  if (store.settings.stale_threshold_days) {
    staleThresholdDays.value = Number(store.settings.stale_threshold_days)
  }

  /* Hydrate ligatures preference */
  const storedLigatures =
    store.settings.code_ligatures ??
    localStorage.getItem(LIGATURES_KEY)
  if (storedLigatures === 'false') {
    ligaturesEnabled.value = false
  }
  applyLigatures(ligaturesEnabled.value)

  /* Hydrate high contrast preference */
  const storedHighContrast =
    store.settings.high_contrast ??
    localStorage.getItem(HIGH_CONTRAST_KEY)
  if (storedHighContrast === 'true') {
    highContrastEnabled.value = true
  }
  applyHighContrast(highContrastEnabled.value)
})

async function saveSettings() {
  saving.value = true
  try {
    const currentPoll = store.settings.poll_interval_seconds
    if (currentPoll !== String(pollInterval.value)) {
      await store.updateSetting('poll_interval_seconds', String(pollInterval.value))
    }
    const currentEditor = store.settings.editor_command
    if (currentEditor !== editorCommand.value) {
      await store.updateSetting('editor_command', editorCommand.value)
    }
    const currentStale = store.settings.stale_threshold_days
    if (currentStale !== String(staleThresholdDays.value)) {
      await store.updateSetting('stale_threshold_days', String(staleThresholdDays.value))
    }
    saved.value = true
    setTimeout(() => { saved.value = false }, 2000)
  } finally {
    saving.value = false
  }
}
</script>

<template>
  <div class="settings-view">
    <h2 class="section-title">Settings</h2>

    <div class="settings-card">
      <div class="setting-row">
        <div class="setting-info">
          <label class="setting-label">Sync Interval</label>
          <p class="setting-description">How often to fetch new PR data from GitHub (in seconds)</p>
        </div>
        <div class="setting-control">
          <input
            v-model.number="pollInterval"
            type="number"
            min="60"
            max="3600"
            step="60"
            class="input-number"
          />
          <span class="unit">seconds</span>
        </div>
      </div>

      <div class="setting-row">
        <div class="setting-info">
          <label class="setting-label">Editor Command</label>
          <p class="setting-description">Command used to open files and directories (e.g. <code>code</code>, <code>cursor</code>, <code>zed</code>)</p>
        </div>
        <div class="setting-control">
          <input
            v-model="editorCommand"
            type="text"
            placeholder="code"
            class="input-text"
          />
        </div>
      </div>

      <div class="setting-row">
        <div class="setting-info">
          <label class="setting-label">Stale Threshold</label>
          <p class="setting-description">Pull requests older than this many days are considered stale and surfaced in the dashboard</p>
        </div>
        <div class="setting-control">
          <input
            v-model.number="staleThresholdDays"
            type="number"
            min="1"
            max="365"
            step="1"
            class="input-number"
          />
          <span class="unit">days</span>
        </div>
      </div>

      <div class="setting-row">
        <div class="setting-info">
          <label class="setting-label">Theme</label>
          <p class="setting-description">Application colour scheme — System follows your OS preference</p>
        </div>
        <div class="setting-control">
          <div class="density-toggle">
            <button
              class="density-option"
              :class="{ active: themeMode === 'dark' }"
              @click="setThemeMode('dark')"
            >
              Dark
            </button>
            <button
              class="density-option"
              :class="{ active: themeMode === 'light' }"
              @click="setThemeMode('light')"
            >
              Light
            </button>
            <button
              class="density-option"
              :class="{ active: themeMode === 'system' }"
              @click="setThemeMode('system')"
            >
              System
            </button>
          </div>
        </div>
      </div>

      <div class="setting-row">
        <div class="setting-info">
          <label class="setting-label">Accent Colour</label>
          <p class="setting-description">Choose a colour that flows through buttons, links, and highlights</p>
        </div>
        <div class="setting-control">
          <div class="accent-swatches">
            <button
              v-for="preset in ACCENT_PRESETS"
              :key="preset.name"
              class="accent-swatch"
              :class="{ selected: accentName === preset.name }"
              :style="{ '--swatch-color': `hsl(${preset.hue}, ${preset.sat}%, ${preset.light}%)` }"
              :title="preset.name"
              :aria-label="`Accent colour: ${preset.name}`"
              @click="setAccent(preset.name)"
            />
          </div>
        </div>
      </div>

      <div class="setting-row">
        <div class="setting-info">
          <label class="setting-label">Density</label>
          <p class="setting-description">Control layout spacing — compact mode reduces padding by 40 %</p>
        </div>
        <div class="setting-control">
          <div class="density-toggle">
            <button
              class="density-option"
              :class="{ active: density === 'comfortable' }"
              @click="setDensity('comfortable')"
            >
              Comfortable
            </button>
            <button
              class="density-option"
              :class="{ active: density === 'compact' }"
              @click="setDensity('compact')"
            >
              Compact
            </button>
          </div>
        </div>
      </div>

      <div class="setting-row">
        <div class="setting-info">
          <label class="setting-label">Code Ligatures</label>
          <p class="setting-description">Enable font ligatures for operators in the diff viewer (<code>=></code>, <code>!==</code>, etc.)</p>
        </div>
        <div class="setting-control">
          <button
            class="toggle-switch"
            :class="{ on: ligaturesEnabled }"
            role="switch"
            :aria-checked="ligaturesEnabled"
            @click="toggleLigatures"
          >
            <span class="toggle-thumb" />
          </button>
        </div>
      </div>

      <div class="setting-row">
        <div class="setting-info">
          <label class="setting-label">High Contrast</label>
          <p class="setting-description">Increase border contrast, remove transparency, and brighten muted text for improved legibility</p>
        </div>
        <div class="setting-control">
          <button
            class="toggle-switch"
            :class="{ on: highContrastEnabled }"
            role="switch"
            :aria-checked="highContrastEnabled"
            @click="toggleHighContrast"
          >
            <span class="toggle-thumb" />
          </button>
        </div>
      </div>
    </div>

    <div class="settings-actions">
      <button class="btn-save" @click="saveSettings">
        {{ saved ? 'Saved!' : 'Save Settings' }}
      </button>
    </div>

    <div class="templates-section">
      <TemplateManager />
    </div>

    <div class="label-rules-section">
      <LabelRulesManager />
    </div>

    <section class="about-section">
      <h3 class="about-title">About</h3>
      <p class="about-text">Fuse v0.1.0</p>
      <p class="about-text">A desktop app for triaging, briefing, and reviewing GitHub pull requests.</p>
    </section>
  </div>
</template>

<style scoped>
.settings-view {
  width: 100%;
}

.section-title {
  font-size: var(--text-heading-size);
  font-weight: var(--text-heading-weight);
  letter-spacing: var(--text-heading-tracking);
  line-height: var(--text-heading-leading);
  margin-bottom: var(--space-6);
  color: var(--color-text-primary);
}

.settings-card {
  background: var(--color-surface-panel);
  border: 1px solid var(--color-border-default);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-card);
  overflow: hidden;
  margin-bottom: var(--space-6);
}

.setting-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: var(--space-5);
  border-bottom: 1px solid var(--color-border-default);
  transition: background var(--transition-fast);
}

.setting-row:last-child {
  border-bottom: none;
}

.setting-row:hover {
  background: var(--color-surface-hover);
}

.setting-label {
  font-size: var(--text-body-size);
  font-weight: var(--text-subheading-weight);
  margin-bottom: var(--space-1);
  display: block;
  color: var(--color-text-primary);
}

.setting-description {
  font-size: var(--text-caption-size);
  font-weight: var(--text-caption-weight);
  color: var(--color-text-muted);
}

.setting-control {
  display: flex;
  align-items: center;
  gap: var(--space-2);
}

.input-number {
  width: 100px;
  text-align: right;
  background: var(--color-surface-input);
  border: 1px solid var(--color-border-default);
  border-radius: var(--radius-md);
  padding: var(--space-2) var(--space-3);
  color: var(--color-text-primary);
  font-size: 13px;
  transition: border-color var(--transition-fast);
}

.input-number:focus {
  border-color: var(--color-border-focus);
  box-shadow: 0 0 0 2px var(--color-accent-muted);
  outline: none;
}

.input-text {
  width: 160px;
  background: var(--color-surface-input);
  border: 1px solid var(--color-border-default);
  border-radius: var(--radius-md);
  padding: var(--space-2) var(--space-3);
  color: var(--color-text-primary);
  font-size: 13px;
  font-family: var(--font-mono);
  transition: border-color var(--transition-fast);
}

.input-text:focus {
  border-color: var(--color-border-focus);
  box-shadow: 0 0 0 2px var(--color-accent-muted);
  outline: none;
}

.input-text::placeholder {
  color: var(--color-text-muted);
}

.setting-description code {
  font-size: 12px;
  font-family: var(--font-mono);
  background: var(--color-surface-raised);
  padding: 1px var(--space-1);
  border-radius: var(--radius-sm);
  color: var(--color-text-secondary);
}

.input-select {
  min-width: 120px;
  background: var(--color-surface-input);
  border: 1px solid var(--color-border-default);
  border-radius: var(--radius-md);
  padding: var(--space-2) var(--space-3);
  color: var(--color-text-primary);
  font-size: 13px;
  transition: border-color var(--transition-fast);
}

.input-select:focus {
  border-color: var(--color-border-focus);
  box-shadow: 0 0 0 2px var(--color-accent-muted);
  outline: none;
}

.unit {
  font-size: 13px;
  color: var(--color-text-muted);
}

.settings-actions {
  margin-bottom: var(--space-8);
}

.btn-save {
  background: var(--color-accent);
  color: var(--color-text-inverse);
  font-weight: 600;
  padding: var(--space-3) var(--space-6);
  border-radius: var(--radius-md);
  border: none;
  cursor: pointer;
  transition: background var(--transition-fast), transform var(--transition-fast);
}

.btn-save:hover {
  background: var(--color-accent-hover);
}

.btn-save:active {
  transform: scale(0.97);
}

.btn-save:focus-visible {
  box-shadow: 0 0 0 2px var(--color-accent-muted);
  outline: none;
}

.templates-section {
  margin-bottom: var(--space-8);
}

.label-rules-section {
  margin-bottom: var(--space-8);
}

.about-section {
  padding: var(--space-5);
  background: var(--color-surface-panel);
  border: 1px solid var(--color-border-default);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-card);
}

.about-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--color-text-secondary);
  margin-bottom: var(--space-3);
}

.about-text {
  font-size: 13px;
  color: var(--color-text-muted);
  margin-bottom: var(--space-1);
}

/* Density toggle — segmented control */
.density-toggle {
  display: flex;
  background: var(--color-surface-input);
  border: 1px solid var(--color-border-default);
  border-radius: var(--radius-md);
  overflow: hidden;
}

.density-option {
  padding: var(--space-2) var(--space-3);
  font-size: 13px;
  font-weight: 500;
  color: var(--color-text-muted);
  background: transparent;
  border: none;
  cursor: pointer;
  transition: all var(--transition-fast);
  white-space: nowrap;
}

.density-option.active {
  background: var(--color-accent);
  color: var(--color-text-inverse);
}

.density-option:not(.active):hover {
  color: var(--color-text-primary);
  background: var(--color-surface-hover);
}

/* Toggle switch — on/off control */
.toggle-switch {
  position: relative;
  width: 40px;
  height: 22px;
  border-radius: var(--radius-full);
  background: var(--color-surface-input);
  border: 1px solid var(--color-border-default);
  cursor: pointer;
  transition: background var(--transition-fast), border-color var(--transition-fast);
  padding: 0;
}

.toggle-switch.on {
  background: var(--color-accent);
  border-color: var(--color-accent);
}

.toggle-thumb {
  position: absolute;
  top: 2px;
  left: 2px;
  width: 16px;
  height: 16px;
  border-radius: var(--radius-full);
  background: var(--color-text-primary);
  transition: transform var(--transition-fast);
}

.toggle-switch.on .toggle-thumb {
  transform: translateX(18px);
  background: var(--color-text-inverse);
}

/* Accent colour swatches */
.accent-swatches {
  display: flex;
  gap: var(--space-2);
}

.accent-swatch {
  width: 28px;
  height: 28px;
  border-radius: var(--radius-full);
  background: var(--swatch-color);
  border: 2px solid transparent;
  cursor: pointer;
  padding: 0;
  transition: transform var(--transition-fast), border-color var(--transition-fast), box-shadow var(--transition-fast);
  position: relative;
}

.accent-swatch:hover {
  transform: scale(1.15);
}

.accent-swatch.selected {
  border-color: var(--color-text-primary);
  box-shadow: 0 0 0 2px var(--swatch-color);
}

/* Check mark on selected swatch */
.accent-swatch.selected::after {
  content: '';
  position: absolute;
  left: 7px;
  top: 3px;
  width: 6px;
  height: 11px;
  border: solid #fff;
  border-width: 0 2px 2px 0;
  transform: rotate(45deg);
}

.accent-swatch:focus-visible {
  outline: 2px solid var(--color-border-focus);
  outline-offset: 2px;
}
</style>
