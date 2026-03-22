<script setup lang="ts">
import { computed, ref } from 'vue'
import { SFileTree, SSectionHeader } from '@stuntrocket/ui'
import type { SFileTreeItem } from '@stuntrocket/ui'
import type { DiffFile } from '../types'

const props = defineProps<{
  files: DiffFile[]
}>()

const emit = defineEmits<{
  (e: 'select-file', path: string): void
}>()

/**
 * Build an SFileTreeItem hierarchy from the flat list of DiffFile paths.
 * Each file is marked with status 'modified' so that the library renders
 * a colour-coded change indicator.
 */
function buildTreeItems(files: DiffFile[]): SFileTreeItem[] {
  const root: SFileTreeItem[] = []

  for (const file of files) {
    const parts = file.path.split('/')
    let current = root

    for (let i = 0; i < parts.length; i++) {
      const part = parts[i]
      const isLast = i === parts.length - 1
      const fullPath = parts.slice(0, i + 1).join('/')

      let existing = current.find(n => n.path === fullPath)
      if (!existing) {
        existing = {
          name: part,
          path: fullPath,
          type: isLast ? 'file' : 'directory',
          children: isLast ? undefined : [],
          status: isLast ? 'modified' : undefined,
        }
        current.push(existing)
      }

      if (!isLast) {
        if (!existing.children) {
          existing.children = []
        }
        current = existing.children
      }
    }
  }

  return root
}

const treeItems = computed(() => buildTreeItems(props.files))

/**
 * Maintain expanded directory paths. All directories start expanded
 * to match the previous default behaviour (collapsed was opt-in).
 */
const expandedPaths = ref<string[]>([])
const initialised = ref(false)

/** Collect every directory path so we can default them all to expanded. */
function collectDirPaths(items: SFileTreeItem[]): string[] {
  const paths: string[] = []
  for (const item of items) {
    if (item.type === 'directory') {
      paths.push(item.path)
      if (item.children) {
        paths.push(...collectDirPaths(item.children))
      }
    }
  }
  return paths
}

/** Expanded paths: all directories expanded on first render, then user-driven. */
const expandedPathsComputed = computed(() => {
  if (!initialised.value && treeItems.value.length > 0) {
    return collectDirPaths(treeItems.value)
  }
  return expandedPaths.value
})

function handleToggle(path: string) {
  if (!initialised.value) {
    initialised.value = true
    expandedPaths.value = collectDirPaths(treeItems.value)
  }
  const current = [...expandedPaths.value]
  const idx = current.indexOf(path)
  if (idx >= 0) {
    current.splice(idx, 1)
  } else {
    current.push(path)
  }
  expandedPaths.value = current
}

function handleSelect(path: string) {
  emit('select-file', path)
}
</script>

<template>
  <div class="diff-file-tree">
    <SSectionHeader title="Files" />
    <SFileTree
      :items="treeItems"
      :expanded-paths="expandedPathsComputed"
      @select="handleSelect"
      @toggle="handleToggle"
    />
  </div>
</template>

<style scoped>
.diff-file-tree {
  display: flex;
  flex-direction: column;
  overflow-y: auto;
}
</style>
