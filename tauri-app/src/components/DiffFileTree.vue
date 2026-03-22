<script setup lang="ts">
import { computed, reactive } from 'vue'
import { SListRow, SSectionHeader } from '@stuntrocket/ui'
import type { DiffFile } from '../types'

const props = defineProps<{
  files: DiffFile[]
}>()

const emit = defineEmits<{
  (e: 'select-file', path: string): void
}>()

interface TreeNode {
  name: string
  fullPath: string
  isFile: boolean
  additions: number
  deletions: number
  children: TreeNode[]
  depth: number
}

/** Track which directory paths are collapsed */
const collapsed = reactive<Record<string, boolean>>({})

function buildTree(files: DiffFile[]): TreeNode[] {
  const root: TreeNode[] = []

  for (const file of files) {
    const parts = file.path.split('/')
    let current = root

    for (let i = 0; i < parts.length; i++) {
      const part = parts[i]
      const isLast = i === parts.length - 1
      const fullPath = parts.slice(0, i + 1).join('/')

      let existing = current.find(n => n.name === part)
      if (!existing) {
        existing = {
          name: part,
          fullPath,
          isFile: isLast,
          additions: isLast ? file.additions : 0,
          deletions: isLast ? file.deletions : 0,
          children: [],
          depth: i,
        }
        current.push(existing)
      }

      if (!isLast) {
        existing.additions += file.additions
        existing.deletions += file.deletions
        current = existing.children
      }
    }
  }

  return root
}

/** Flatten the tree into a visible list, respecting collapsed state */
function flattenTree(nodes: TreeNode[]): TreeNode[] {
  const result: TreeNode[] = []
  for (const node of nodes) {
    result.push(node)
    if (!node.isFile && !collapsed[node.fullPath] && node.children.length > 0) {
      result.push(...flattenTree(node.children))
    }
  }
  return result
}

const flatNodes = computed(() => {
  const tree = buildTree(props.files)
  return flattenTree(tree)
})

function toggleDir(node: TreeNode) {
  if (node.isFile) {
    emit('select-file', node.fullPath)
  } else {
    collapsed[node.fullPath] = !collapsed[node.fullPath]
  }
}

function isExpanded(node: TreeNode): boolean {
  return !collapsed[node.fullPath]
}


</script>

<template>
  <div class="diff-file-tree">
    <SSectionHeader title="Files" />
    <div class="tree-list">
      <SListRow
        v-for="node in flatNodes"
        :key="node.fullPath"
        :class="{ 'tree-file': node.isFile, 'tree-dir': !node.isFile }"
        :style="{ paddingLeft: `${node.depth * 12 + 8}px` }"
        @click="toggleDir(node)"
      >
        <template #default>
          <span v-if="!node.isFile" class="tree-arrow" :class="{ expanded: isExpanded(node) }">&#9654;</span>
          <span v-else class="tree-file-icon">&#9675;</span>
          <span class="tree-name">{{ node.name }}</span>
        </template>
        <template #actions>
          <span class="tree-stats">
            <span v-if="node.additions > 0" class="tree-add">+{{ node.additions }}</span>
            <span v-if="node.deletions > 0" class="tree-del">-{{ node.deletions }}</span>
          </span>
        </template>
      </SListRow>
    </div>
  </div>
</template>

<style scoped>
.diff-file-tree {
  display: flex;
  flex-direction: column;
  overflow-y: auto;
}

.tree-list {
  display: flex;
  flex-direction: column;
}

.tree-arrow {
  font-size: 8px;
  color: var(--color-text-muted);
  transition: transform var(--transition-fast);
  display: inline-block;
  width: 12px;
  text-align: center;
  flex-shrink: 0;
}

.tree-arrow.expanded {
  transform: rotate(90deg);
}

.tree-file-icon {
  font-size: 8px;
  color: var(--color-text-muted);
  width: 12px;
  text-align: center;
  flex-shrink: 0;
}

.tree-name {
  color: var(--color-text-secondary);
  overflow: hidden;
  text-overflow: ellipsis;
  font-size: 12px;
}

.tree-file .tree-name {
  color: var(--color-text-primary);
}

.tree-stats {
  display: flex;
  gap: var(--space-1);
  font-family: var(--font-mono);
  font-size: 11px;
  flex-shrink: 0;
}

.tree-add {
  color: var(--color-status-success);
}

.tree-del {
  color: var(--color-status-danger);
}
</style>
