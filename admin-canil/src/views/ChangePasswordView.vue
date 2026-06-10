<template>
  <div class="password-container">
    <div class="password-box">
      <h2>⚠️ Troca de Senha Obrigatória</h2>
      <p>Como este é o seu primeiro acesso, você precisa definir uma nova senha segura.</p>
      
      <form @submit.prevent="handleChangePassword">
        <div class="input-group">
          <label for="new_password">Nova Senha</label>
          <div class="password-input-wrapper">
            <input 
              :type="showPassword ? 'text' : 'password'" 
              id="new_password" 
              v-model="newPassword" 
              required 
              placeholder="Digite a nova senha (mínimo 10 caracteres)"
              minlength="10"
            />
            <button type="button" class="btn-toggle-password" @click="showPassword = !showPassword" title="Mostrar/Ocultar senha">
              <span style="font-size: 0.8rem; font-weight: bold; color: #475569;">{{ showPassword ? 'OCULTAR' : 'MOSTRAR' }}</span>
            </button>
          </div>
          <p class="password-hint">
            A senha deve ter no mínimo 10 caracteres, 1 número e 1 caractere especial (ex: @, !, #).
          </p>
          <PasswordStrength :password="newPassword" />
        </div>
        
        <div class="input-group">
          <label for="confirm_password">Confirme a Nova Senha</label>
          <div class="password-input-wrapper">
            <input 
              :type="showPassword ? 'text' : 'password'" 
              id="confirm_password" 
              v-model="confirmPassword" 
              required 
              placeholder="Repita a nova senha"
              minlength="10"
            />
          </div>
        </div>

        <div v-if="errorMessage" class="error-message">
          {{ errorMessage }}
        </div>

        <div v-if="successMessage" class="success-message">
          {{ successMessage }}
        </div>

        <button type="submit" :disabled="isLoading">
          {{ isLoading ? 'Atualizando...' : 'Atualizar Senha e Entrar' }}
        </button>
      </form>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import api from '../services/api'
import PasswordStrength from '../components/PasswordStrength.vue'

const router = useRouter()
const newPassword = ref('')
const confirmPassword = ref('')
const errorMessage = ref('')
const successMessage = ref('')
const isLoading = ref(false)
const showPassword = ref(false)

const handleChangePassword = async () => {
  if (newPassword.value.length < 10) {
    errorMessage.value = 'A senha deve ter pelo menos 10 caracteres.'
    return
  }
  if (!/\d/.test(newPassword.value)) {
    errorMessage.value = 'A senha deve conter pelo menos um número.'
    return
  }
  if (!/[^a-zA-Z0-9]/.test(newPassword.value)) {
    errorMessage.value = 'A senha deve conter pelo menos um caractere especial.'
    return
  }
  if (newPassword.value !== confirmPassword.value) {
    errorMessage.value = 'As senhas não coincidem.'
    return
  }

  isLoading.value = true
  errorMessage.value = ''
  
  try {
    await api.patch('/auth/password', {
      new_password: newPassword.value
    })
    
    successMessage.value = 'Senha atualizada com sucesso! Redirecionando...'
    
    setTimeout(() => {
      router.push('/dashboard')
    }, 1500)

  } catch (error: any) {
    if (error.response && error.response.data && error.response.data.error) {
      errorMessage.value = error.response.data.error
    } else {
      errorMessage.value = 'Erro ao atualizar a senha.'
    }
  } finally {
    isLoading.value = false
  }
}
</script>

<style scoped>
.password-container {
  display: flex;
  justify-content: center;
  align-items: center;
  min-height: 100vh;
  background-color: #f3f4f6;
  font-family: 'Segoe UI', sans-serif;
}

.password-box {
  background: white;
  padding: 2.5rem;
  border-radius: 8px;
  box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
  width: 100%;
  max-width: 450px;
  text-align: center;
  border-top: 4px solid #eab308;
}

h2 { margin: 0 0 0.5rem 0; color: #1f2937; }
p { color: #6b7280; margin-bottom: 2rem; font-size: 0.95rem; }

.input-group { margin-bottom: 1.5rem; text-align: left; }
label { display: block; margin-bottom: 0.5rem; color: #374151; font-weight: 500; }
input { width: 100%; padding: 0.75rem; border: 1px solid #d1d5db; border-radius: 4px; font-size: 1rem; box-sizing: border-box; }
input:focus { outline: none; border-color: #22c55e; box-shadow: 0 0 0 2px rgba(34, 197, 94, 0.2); }

button {
  width: 100%; padding: 0.75rem; background-color: #22c55e; color: white; border: none; border-radius: 4px; font-size: 1rem; font-weight: bold; cursor: pointer; transition: background-color 0.2s;
}
button:hover:not(:disabled) { background-color: #16a34a; }
button:disabled { background-color: #9ca3af; cursor: not-allowed; }

.error-message { color: #dc2626; background-color: #fee2e2; padding: 0.75rem; border-radius: 4px; margin-bottom: 1rem; font-size: 0.9rem; }
.success-message { color: #16a34a; background-color: #dcfce3; padding: 0.75rem; border-radius: 4px; margin-bottom: 1rem; font-size: 0.9rem; font-weight: bold; }

.password-input-wrapper {
  position: relative;
  display: block;
}
.password-input-wrapper input {
  width: 100%;
  box-sizing: border-box;
  padding-right: 70px;
}
.btn-toggle-password {
  position: absolute;
  right: 10px;
  top: 50%;
  transform: translateY(-50%);
  background: transparent !important;
  border: none;
  cursor: pointer;
  padding: 0 !important;
  width: auto !important;
  display: flex;
  align-items: center;
  justify-content: center;
}
.btn-toggle-password:hover {
  background: transparent !important;
  opacity: 0.7;
}
.password-hint {
  font-size: 0.8rem;
  color: #6b7280;
  margin-top: 4px;
  margin-bottom: 8px;
}
</style>