<script setup lang="ts">
import ThemePanel from '../ThemePanel.vue'
import IconPanel from '../IconPanel.vue'
</script>

<template>
  <div class="h-screen w-screen flex flex-col bg-hoi4-dark overflow-hidden">
    <slot />
    <ThemePanel />
    <IconPanel />
  </div>
</template>
