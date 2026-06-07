<!-- ========================================== -->
<!-- 🏡 A CASCA DO APLICATIVO (App.vue) -->
<!-- ========================================== -->
<!-- Tudo começa aqui. Essa tag <RouterView /> é como se fosse uma "TV": 
ela vai sintonizar no "canal" (página) certo dependendo da URL que o usuário digitar! -->
<template>
  <RouterView />
</template>

<script setup lang="ts">
import { RouterView } from 'vue-router'
</script>

<!-- Nossos estilos globais (CSS que afeta o site todinho) -->
<style>
body {
  margin: 0;
  padding: 0;
  background-color: #f8fafc;
}

* {
  box-sizing: border-box;
  overflow-wrap: break-word;
}
</style>