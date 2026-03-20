<script setup lang="ts">
import { ref, computed } from 'vue'

const props = withDefaults(defineProps<{
  username: string
  size?: number
}>(), {
  size: 20,
})

const imgFailed = ref(false)

const avatarUrl = computed(() =>
  `https://github.com/${props.username}.png?size=${props.size * 2}`
)

const initial = computed(() =>
  (props.username?.[0] ?? '?').toUpperCase()
)

const sizeStyle = computed(() => ({
  width: `${props.size}px`,
  height: `${props.size}px`,
  fontSize: `${Math.round(props.size * 0.5)}px`,
}))

/** Generate a deterministic hue from the username. */
const fallbackColour = computed(() => {
  let hash = 0
  for (const ch of props.username) {
    hash = ch.charCodeAt(0) + ((hash << 5) - hash)
  }
  const hue = Math.abs(hash) % 360
  return `hsl(${hue}, 50%, 40%)`
})

function handleError() {
  imgFailed.value = true
}
</script>

<template>
  <img
    v-if="!imgFailed"
    :src="avatarUrl"
    :alt="username"
    loading="lazy"
    class="author-avatar"
    :style="sizeStyle"
    @error="handleError"
  />
  <span
    v-else
    class="author-avatar author-avatar--fallback"
    :style="{ ...sizeStyle, backgroundColor: fallbackColour }"
  >{{ initial }}</span>
</template>

<style scoped>
.author-avatar {
  border-radius: var(--radius-full);
  border: 1px solid var(--color-border-default);
  display: inline-block;
  flex-shrink: 0;
  vertical-align: middle;
  object-fit: cover;
}

.author-avatar--fallback {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  color: white;
  font-weight: 600;
  line-height: 1;
}
</style>
