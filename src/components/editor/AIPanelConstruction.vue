<script setup lang="ts">
const constructionText = 'AI功能施工中...'
</script>

<template>
  <div class="h-full flex items-center justify-center p-6">
    <div class="flex flex-col items-center justify-center text-center gap-4">
      <svg
        class="w-20 h-20 text-hoi4-accent"
        viewBox="0 0 64 64"
        fill="none"
        xmlns="http://www.w3.org/2000/svg"
        aria-hidden="true"
      >
        <rect x="10" y="14" width="44" height="36" rx="6" stroke="currentColor" stroke-width="3" />
        <path d="M24 24H40" stroke="currentColor" stroke-width="3" stroke-linecap="round" />
        <path d="M24 32H40" stroke="currentColor" stroke-width="3" stroke-linecap="round" />
        <path d="M24 40H34" stroke="currentColor" stroke-width="3" stroke-linecap="round" />
        <path d="M32 8V14" stroke="currentColor" stroke-width="3" stroke-linecap="round" />
        <path d="M20 54L16 58" stroke="currentColor" stroke-width="3" stroke-linecap="round" />
        <path d="M44 54L48 58" stroke="currentColor" stroke-width="3" stroke-linecap="round" />
      </svg>
      <p class="text-base font-semibold text-hoi4-text">
        {{ constructionText }}
      </p>
    </div>
  </div>
</template>
