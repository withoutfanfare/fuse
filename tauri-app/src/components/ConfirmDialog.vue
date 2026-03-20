<script setup lang="ts">
import { SConfirmDialog, useConfirm } from '@stuntrocket/ui'

const { isOpen, options, handleConfirm, handleCancel } = useConfirm()
</script>

<template>
  <SConfirmDialog
    :open="isOpen"
    :title="options.title"
    :message="options.message"
    :confirm-label="options.confirmLabel"
    :cancel-label="options.cancelLabel"
    :danger="options.danger ?? false"
    @confirm="handleConfirm"
    @cancel="handleCancel"
    @close="handleCancel"
  />
</template>
