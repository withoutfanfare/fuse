<script setup lang="ts">
import { computed } from 'vue'
import { SAvatar } from '@stuntrocket/ui'

const props = withDefaults(defineProps<{
  username: string
  size?: number
}>(), {
  size: 20,
})

const avatarUrl = computed(() =>
  `https://github.com/${props.username}.png?size=${props.size * 2}`
)

/** Map numeric px size to SAvatar's size tokens */
const sizeToken = computed(() => {
  if (props.size <= 20) return 'sm'
  if (props.size <= 32) return 'md'
  return 'lg'
})
</script>

<template>
  <SAvatar :name="username" :src="avatarUrl" :size="sizeToken" />
</template>
