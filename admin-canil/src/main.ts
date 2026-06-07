import { createApp } from 'vue'
import { createPinia } from 'pinia'

// ==========================================
// 🚀 INÍCIO DO FRONTEND: O Painel de Administração ganha vida!
// ==========================================

import App from './App.vue'     // Nossa tela principal (a casca onde tudo vai aparecer)
import router from './router'   // O "GPS" que nos leva para a tela de Login ou Dashboard

// Cria a aplicação Vue. Pense nisso como ligar o motor do carro! 🚗
const app = createApp(App)

// Pinia é a memória do site (guarda o estado de login, preferências, etc)
app.use(createPinia())

// E o router guia a gente pelas páginas
app.use(router)

// Pluga tudo isso na tela (lá na div #app do index.html). E voilà! ✨
app.mount('#app')
