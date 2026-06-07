import './style.css'
import { createApp } from 'vue'
import App from './App.vue'
import router from './router'

// ==========================================
// 🚀 INÍCIO DO SITE: O portal público nasce aqui!
// ==========================================
// Assim como no painel administrativo, a gente cria a "instância" (o motor) do Vue
// e o "Router" (nosso mapa de navegação).

const app = createApp(App)

app.use(router)

// Pluga tudo na div #app do index.html. Bem vindo ao site do Canil!
app.mount('#app')
