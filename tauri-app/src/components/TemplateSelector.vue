<script setup lang="ts">
import { ref, onMounted, onUnmounted, nextTick } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { FileText } from 'lucide-vue-next'
import type { ReviewTemplate } from '../types'

const emit = defineEmits<{
  'select-template': [body: string]
}>()

const templates = ref<ReviewTemplate[]>([])
const open = ref(false)
const triggerRef = ref<HTMLButtonElement | null>(null)
const dropdownStyle = ref<Record<string, string>>({})

onMounted(async () => {
  try {
    templates.value = await invoke<ReviewTemplate[]>('list_templates')
  } catch {
    templates.value = []
  }
})

function selectTemplate(template: ReviewTemplate) {
  emit('select-template', template.body)
  open.value = false
}

function positionDropdown() {
  if (!triggerRef.value) return
  const rect = triggerRef.value.getBoundingClientRect()
  dropdownStyle.value = {
    position: 'fixed',
    bottom: `${window.innerHeight - rect.top + 8}px`,
    right: `${window.innerWidth - rect.right}px`,
  }
}

function toggle() {
  open.value = !open.value
  if (open.value) {
    nextTick(positionDropdown)
  }
}

function handleClickOutside(e: MouseEvent) {
  if (!open.value) return
  const target = e.target as HTMLElement
  if (triggerRef.value?.contains(target)) return
  open.value = false
}

onMounted(() => document.addEventListener('mousedown', handleClickOutside))
onUnmounted(() => document.removeEventListener('mousedown', handleClickOutside))
</script>

<template>
  <div class="template-selector">
    <button ref="triggerRef" class="btn-template-trigger" @click="toggle" title="Quick review templates">
      <FileText :size="14" />
      Template
    </button>
    <Teleport to="body">
      <Transition name="dropdown-fade">
        <div v-if="open" class="template-dropdown" :style="dropdownStyle">
          <div
            v-if="templates.length === 0"
            class="template-empty"
          >
            No templates configured
          </div>
          <button
            v-for="tpl in templates"
            :key="tpl.id"
            class="template-option"
            @click="selectTemplate(tpl)"
          >
            <span class="template-name">{{ tpl.name }}</span>
            <span class="template-preview">{{ tpl.body.slice(0, 60) }}{{ tpl.body.length > 60 ? '...' : '' }}</span>
          </button>
        </div>
      </Transition>
    </Teleport>
  </div>
</template>

<style scoped>
.template-selector {
  position: relative;
  display: inline-block;
}

.btn-template-trigger {
  display: inline-flex;
  align-items: center;
  gap: var(--space-1);
  background: var(--color-surface-raised);
  color: var(--color-text-secondary);
  font-size: 12px;
  font-weight: 500;
  padding: var(--space-1) var(--space-2);
  border-radius: var(--radius-md);
  border: 1px solid var(--color-border-default);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.btn-template-trigger:hover {
  background: var(--color-surface-hover);
  color: var(--color-text-primary);
  border-color: var(--color-border-hover);
}

.btn-template-trigger:focus-visible {
  box-shadow: 0 0 0 2px var(--color-accent-muted);
  outline: none;
}
</style>

<style>
/* Unscoped — teleported dropdown renders outside the component tree */
.template-dropdown {
  min-width: 280px;
  max-width: 360px;
  background: rgba(26, 24, 22, 0.95);
  border: 1px solid var(--color-border-default);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-overlay);
  overflow: hidden;
  z-index: 9999;
}

.template-empty {
  padding: var(--space-4);
  text-align: center;
  font-size: 13px;
  color: var(--color-text-muted);
}

.template-option {
  display: flex;
  flex-direction: column;
  gap: var(--space-0-5);
  width: 100%;
  padding: var(--space-3) var(--space-4);
  background: none;
  border: none;
  border-bottom: 1px solid rgba(255, 255, 255, 0.04);
  cursor: pointer;
  text-align: left;
  transition: background var(--transition-fast);
}

.template-option:last-child {
  border-bottom: none;
}

.template-option:hover {
  background: var(--color-surface-hover);
}

.template-name {
  font-size: 13px;
  font-weight: 600;
  color: var(--color-text-primary);
}

.template-preview {
  font-size: 12px;
  color: var(--color-text-muted);
  line-height: 1.4;
}

/* Dropdown fade transition */
.dropdown-fade-enter-active {
  transition: opacity 150ms ease, transform 150ms ease;
}

.dropdown-fade-leave-active {
  transition: opacity 100ms ease, transform 100ms ease;
}

.dropdown-fade-enter-from {
  opacity: 0;
  transform: translateY(4px);
}

.dropdown-fade-leave-to {
  opacity: 0;
  transform: translateY(4px);
}
</style>
