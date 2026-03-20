<script setup lang="ts">
import { ref, watch, nextTick, onMounted } from 'vue'

const props = defineProps<{
  tabs: { key: string; label: string }[]
  modelValue: string
}>()

const emit = defineEmits<{
  'update:modelValue': [value: string]
}>()

const tabRefs = ref<HTMLButtonElement[]>([])
const indicatorStyle = ref({ transform: 'translateX(0px)', width: '0px' })

function updateIndicator() {
  const activeIndex = props.tabs.findIndex(t => t.key === props.modelValue)
  if (activeIndex >= 0 && tabRefs.value[activeIndex]) {
    const el = tabRefs.value[activeIndex]
    indicatorStyle.value = {
      transform: `translateX(${el.offsetLeft}px)`,
      width: `${el.offsetWidth}px`,
    }
  }
}

watch(() => props.modelValue, () => {
  nextTick(updateIndicator)
})

onMounted(() => {
  nextTick(updateIndicator)
})

function setTabRef(el: any, index: number) {
  if (el) tabRefs.value[index] = el
}
</script>

<template>
  <div class="tab-bar">
    <div class="tab-bar-inner">
      <button
        v-for="(tab, index) in tabs"
        :key="tab.key"
        :ref="(el) => setTabRef(el, index)"
        class="tab-btn"
        :class="{ active: modelValue === tab.key }"
        @click="emit('update:modelValue', tab.key)"
      >
        {{ tab.label }}
      </button>
      <div class="tab-indicator" :style="indicatorStyle" />
    </div>
  </div>
</template>

<style scoped>
.tab-bar {
  padding: 0;
  margin-bottom: var(--space-5);
  border-bottom: 1px solid var(--color-border-default);
}

.tab-bar-inner {
  position: relative;
  display: flex;
  gap: var(--space-5);
}

.tab-btn {
  position: relative;
  z-index: 1;
  background: none;
  border: none;
  padding: var(--space-2) 0;
  font-size: 13px;
  font-weight: 500;
  color: var(--color-text-muted);
  cursor: pointer;
  transition: color var(--transition-fast);
}

.tab-btn:hover {
  color: var(--color-text-secondary);
}

.tab-btn:active {
  transform: none;
}

.tab-btn.active {
  color: var(--color-text-primary);
}

.tab-btn:focus-visible {
  box-shadow: 0 0 0 2px var(--color-accent-muted);
  outline: none;
}

.tab-indicator {
  position: absolute;
  bottom: -1px;
  left: 0;
  height: 2px;
  background: var(--color-accent);
  border-radius: 1px 1px 0 0;
  transition: transform var(--transition-normal), width var(--transition-normal);
  pointer-events: none;
}
</style>
