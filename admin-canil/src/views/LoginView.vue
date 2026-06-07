<!-- ========================================== -->
<!-- 🚪 A PORTA DE ENTRADA: Tela de Login -->
<!-- ========================================== -->
<!-- Não tem muito segredo: o cara digita o email e a senha, a gente
fala com a API e se tiver tudo certo, abrimos a porta pro Dashboard! -->
<template>
  <div class="login-container">
    <div class="login-box">
      <h2>Acesso ao Painel</h2>
      <p>Gerenciamento do Canil</p>
      
      <form @submit.prevent="handleLogin">
        <div class="input-group">
          <label for="email">Usuário / E-mail</label>
          <input 
            type="text" 
            id="email" 
            v-model="email" 
            required 
            placeholder="Digite seu usuário ou e-mail"
          />
        </div>
        
        <div class="input-group">
          <label for="password">Senha</label>
          <input 
            type="password" 
            id="password" 
            v-model="password" 
            required 
            placeholder="Digite sua senha"
          />
        </div>

        <div v-if="errorMessage" class="error-message">
          {{ errorMessage }}
          <div v-if="remainingAttempts !== null && remainingAttempts > 0" class="attempts-warning">
            Tentativas restantes: <strong>{{ remainingAttempts }}</strong>
          </div>
        </div>

        <button type="submit" :disabled="isLoading">
          {{ isLoading ? 'Entrando...' : 'Entrar' }}
        </button>
        
        <div class="legal-warning">
          ⚠️ <strong>Aviso de Segurança:</strong> Todas as tentativas de acesso, endereço IP e dados do navegador são monitorados e registrados pelo sistema de auditoria para detecção de intrusões. O bloqueio ocorrerá após 5 tentativas falhas.
        </div>
      </form>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import api from '../services/api'

const router = useRouter()
const email = ref('')
const password = ref('')
const errorMessage = ref('')
const remainingAttempts = ref<number | null>(null)
const isLoading = ref(false)

const handleLogin = async () => {
  isLoading.value = true
  errorMessage.value = ''
  remainingAttempts.value = null
  
  try {
    const response = await api.post('/auth/login', {
      email: email.value,
      password: password.value
    })
    
    localStorage.setItem('authToken', response.data.token)
    localStorage.setItem('isMaster', response.data.is_master ? '1' : '0')
    
    if (response.data.is_first_login) {
      router.push('/trocar-senha')
    } else {
      router.push('/dashboard')
    }

  } catch (error: any) {
    if (error.response && error.response.data && error.response.data.error) {
      errorMessage.value = error.response.data.error
      if (error.response.data.remaining_attempts !== undefined) {
        remainingAttempts.value = error.response.data.remaining_attempts
      }
    } else {
      errorMessage.value = 'Erro ao conectar com o servidor.'
    }
  } finally {
    isLoading.value = false
  }
}
</script>

<style scoped>
.login-container {
  display: flex;
  justify-content: center;
  align-items: center;
  min-height: 100vh;
  background-color: #f3f4f6;
  font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
}

.login-box {
  background: white;
  padding: 2.5rem;
  border-radius: 8px;
  box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
  width: 100%;
  max-width: 400px;
  text-align: center;
}

h2 {
  margin: 0;
  color: #1f2937;
}

p {
  color: #6b7280;
  margin-bottom: 2rem;
}

.input-group {
  margin-bottom: 1.5rem;
  text-align: left;
}

label {
  display: block;
  margin-bottom: 0.5rem;
  color: #374151;
  font-weight: 500;
}

input {
  width: 100%;
  padding: 0.75rem;
  border: 1px solid #d1d5db;
  border-radius: 4px;
  font-size: 1rem;
  box-sizing: border-box;
}

input:focus {
  outline: none;
  border-color: #3b82f6;
  box-shadow: 0 0 0 2px rgba(59, 130, 246, 0.2);
}

button {
  width: 100%;
  padding: 0.75rem;
  background-color: #166534;
  color: white;
  border: none;
  border-radius: 4px;
  font-size: 1rem;
  font-weight: bold;
  cursor: pointer;
  transition: background-color 0.2s;
}

button:hover:not(:disabled) {
  background-color: #14532d;
}

button:disabled {
  background-color: #9ca3af;
  cursor: not-allowed;
}

.error-message {
  color: #dc2626;
  background-color: #fee2e2;
  padding: 0.75rem;
  border-radius: 4px;
  margin-bottom: 1rem;
  font-size: 0.9rem;
}

.attempts-warning {
  margin-top: 0.5rem;
  font-size: 0.85rem;
  color: #991b1b;
}

.legal-warning {
  margin-top: 1.5rem;
  font-size: 0.75rem;
  color: #6b7280;
  background: #f9fafb;
  padding: 0.8rem;
  border-radius: 6px;
  border: 1px dashed #d1d5db;
  text-align: left;
}
</style>