<script setup lang="ts">
</script>

<template>
  <div class="title-bar" data-tauri-drag-region>
    <div class="title-bar-content" data-tauri-drag-region>
      <span class="title-text" data-tauri-drag-region></span>
    </div>
  </div>
</template>

<style scoped>
.title-bar {
  height: 28px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  background: var(--color-surface-chrome);
  -webkit-app-region: drag;
  position: relative;
  z-index: 9999;
  flex-shrink: 0;
  /* Padding for native macOS traffic lights */
  padding-left: 78px;
  user-select: none;
}

.title-bar-content {
  flex: 1;
  display: flex;
  align-items: center;
  height: 100%;
  -webkit-app-region: drag;
}

.title-text {
  -webkit-app-region: drag;
  user-select: none;
}
</style>
