<template>
  <RouterView />
</template>

<script setup lang="ts">
import { RouterView } from 'vue-router'
</script>

<style>
body {
  margin: 0;
  padding: 0;
  background-color: #f8fafc;
}

* {
  box-sizing: border-box;
  overflow-wrap: break-word;
  word-break: break-word;
}
</style>