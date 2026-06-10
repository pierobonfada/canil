<!-- ========================================== -->
<!-- 🐶 NOVO INTEGRANTE: Cadastrar Animal -->
<!-- ========================================== -->
<!-- Chegou um novo morador no canil? É aqui que o administrador
preenche o nome, idade, comportamento, faz o upload da foto e joga
o novo pet direto para o estrelato da Home pública! -->
<template>
  <div class="add-animal-container">
    <header class="top-bar">
      <button @click="$router.push('/dashboard')" class="btn-back">
        ← Voltar ao Painel
      </button>
      <h2>🐶 Cadastrar Novo Pet</h2>
    </header>

    <main class="form-content">
      <form @submit.prevent="submitForm" class="animal-form">
        
        <section class="form-section">
          <h3>Identificação</h3>
          <div class="form-group">
            <label>Nome do Animal *</label>
            <input type="text" v-model="form.name" required placeholder="Ex: Frederico" />
          </div>
          
          <div class="form-grid">
            <div class="form-group">
              <label>Espécie *</label>
              <select v-model="form.species" required>
                <option value="" disabled selected>Escolha...</option>
                <option value="Cachorro">Cachorro</option>
                <option value="Gato">Gato</option>
              </select>
            </div>
            <div class="form-group">
              <label>Ano de Nasc. *</label>
              <input type="number" v-model="form.birth_year" required placeholder="Ex: 2024" />
            </div>
          </div>
          
          <div class="form-group">
            <label>Raça</label>
            <input type="text" v-model="form.breed" placeholder="Deixe em branco se Sem Raça Definida (SRD)" />
          </div>
        </section>

        <section class="form-section">
          <h3>Perfil e Saúde</h3>
          <div class="form-grid">
            <div class="form-group">
              <label>Porte *</label>
              <select v-model="form.size" required>
                <option value="" disabled selected>Selecione...</option>
                <option value="Pequeno">Pequeno</option>
                <option value="Médio">Médio</option>
                <option value="Grande">Grande</option>
              </select>
            </div>
            <div class="form-group">
              <label>Independência *</label>
              <select v-model="form.independence" required>
                <option value="" disabled selected>Selecione...</option>
                <option value="Independente">Independente</option>
                <option value="Dependente">Dependente</option>
              </select>
            </div>
          </div>

          <div class="form-grid">
            <div class="form-group">
              <label>Cor Predominante *</label>
              <select v-model="form.predominant_color" required>
                <option value="" disabled selected>Selecione...</option>
                <option value="Branco">Branco</option>
                <option value="Preto">Preto</option>
                <option value="Caramelo">Caramelo</option>
                <option value="Cinza">Cinza</option>
                <option value="Marrom">Marrom</option>
                <option value="Tricolor">Tricolor</option>
                <option value="Bicolor">Bicolor</option>
                <option value="Laranja">Laranja</option>
                <option value="Outra">Outra</option>
              </select>
            </div>
            <div class="form-group">
              <label>Pelagem *</label>
              <select v-model="form.coat_length" required>
                <option value="" disabled selected>Selecione...</option>
                <option value="Curta">Curta</option>
                <option value="Média">Média</option>
                <option value="Longa">Longa</option>
                <option value="Sem Pelo">Sem Pelo</option>
              </select>
            </div>
          </div>
          
          <div class="form-grid">
            <div class="form-group">
              <label>Com Cães *</label>
              <select v-model="form.behavior_dogs" required>
                <option value="" disabled selected>Comportamento...</option>
                <option value="Dócil">Dócil</option>
                <option value="Neutro">Neutro</option>
                <option value="Agressivo">Agressivo</option>
                <option value="Desconhecido">Desconhecido</option>
              </select>
            </div>
            <div class="form-group">
              <label>Com Gatos *</label>
              <select v-model="form.behavior_cats" required>
                <option value="" disabled selected>Comportamento...</option>
                <option value="Dócil">Dócil</option>
                <option value="Neutro">Neutro</option>
                <option value="Agressivo">Agressivo</option>
                <option value="Desconhecido">Desconhecido</option>
              </select>
            </div>
            <div class="form-group">
              <label>Com Humanos *</label>
              <select v-model="form.behavior_humans" required>
                <option value="" disabled selected>Comportamento...</option>
                <option value="Dócil">Dócil</option>
                <option value="Medroso">Medroso</option>
                <option value="Agressivo">Agressivo</option>
                <option value="Desconhecido">Desconhecido</option>
              </select>
            </div>
          </div>

          <div class="form-grid">
            <div class="form-group">
              <label>Vacinado? *</label>
              <select v-model="form.is_vaccinated" required>
                <option value="" disabled selected>Status...</option>
                <option :value="true">Sim</option>
                <option :value="false">Não</option>
              </select>
            </div>
            <div class="form-group">
              <label>Vermifugado? *</label>
              <select v-model="form.is_dewormed" required>
                <option value="" disabled selected>Status...</option>
                <option :value="true">Sim</option>
                <option :value="false">Não</option>
              </select>
            </div>
          </div>

          <div class="form-group">
            <label>Doenças / Observações</label>
            <input type="text" v-model="form.diseases" placeholder="Ex: Nenhuma ou detalhar condições" />
          </div>
        </section>

        <section class="form-section">
          <h3>História e Fotos</h3>
          <div class="form-group">
            <label>Descrição Detalhada *</label>
            <textarea v-model="form.description" required rows="4" placeholder="Conte sobre a personalidade, temperamento e história de resgate..."></textarea>
          </div>
          <div class="form-group">
            <label>Anexar Fotos (A primeira será a capa) *</label>
            <div class="custom-file-upload">
              <input type="file" id="file-upload" multiple accept="image/*" @change="handleFileUpload" required />
              <label for="file-upload" class="file-label">
                📷 Selecionar Imagens
              </label>
              <p class="file-count" v-if="selectedFiles.length > 0">{{ selectedFiles.length }} arquivo(s) selecionado(s)</p>
            </div>
          </div>
        </section>

        <div v-if="isLoading" class="progress-container">
          <div class="progress-bar" :style="{ width: uploadProgress + '%' }"></div>
          <span class="progress-text">{{ uploadProgress }}% - {{ uploadProgress === 100 ? 'Processando fotos no servidor...' : 'Enviando arquivos...' }}</span>
        </div>

        <button type="submit" class="btn-submit" :disabled="isLoading">
          {{ isLoading ? 'Aguarde...' : 'Cadastrar Animal no Sistema' }}
        </button>
      </form>
    </main>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import api from '../services/api'

const router = useRouter()
const isLoading = ref(false)
const uploadProgress = ref(0)
const selectedFiles = ref<File[]>([])

const form = ref({ 
  name: '', species: '', birth_year: null as any, breed: '', 
  is_vaccinated: '' as any, is_dewormed: '' as any, behavior_dogs: '', 
  behavior_cats: '', behavior_humans: '', independence: '', size: '', 
  coat_color: 'Indefinida', predominant_color: '', coat_length: '', description: '', diseases: '' 
})

const handleFileUpload = (e: any) => {
  if (e.target.files) {
    selectedFiles.value = Array.from(e.target.files)
  }
}

const submitForm = async () => {
  const totalSize = selectedFiles.value.reduce((acc, file) => acc + file.size, 0)
  const maxMb = Number(import.meta.env.VITE_MAX_UPLOAD_MB) || 1000
  if (totalSize > maxMb * 1024 * 1024) {
    alert(`O tamanho total das fotos excede o limite de ${maxMb}MB. Reduza a quantidade ou o tamanho das imagens.`)
    return
  }

  isLoading.value = true
  uploadProgress.value = 0
  const formData = new FormData()
  
  Object.entries(form.value).forEach(([key, value]) => {
    if (value !== null && value !== '') formData.append(key, value.toString())
  })
  
  selectedFiles.value.forEach(file => formData.append('photo', file))

  try {
    await api.post('/animais', formData, {
      onUploadProgress: (progressEvent) => {
        if (progressEvent.total) {
          uploadProgress.value = Math.round((progressEvent.loaded * 100) / progressEvent.total)
        }
      }
    })
    alert('Cadastrado com sucesso!')
    router.push('/dashboard')
  } catch (e: any) {
    alert('Erro: ' + (e.response?.data?.error || 'Falha no envio'))
  } finally {
    isLoading.value = false
    uploadProgress.value = 0
  }
}
</script>

<style scoped>
.add-animal-container { min-height: 100vh; background-color: #f8fafc; font-family: 'Segoe UI', Tahoma, sans-serif; padding-bottom: 3rem; }
.top-bar { display: flex; align-items: center; background: linear-gradient(135deg, #166534 0%, #14532d 100%); padding: 1.2rem 1.5rem; color: white; position: sticky; top: 0; z-index: 100; box-shadow: 0 4px 12px rgba(0,0,0,0.1); }
.top-bar h2 { margin: 0 0 0 1rem; font-size: 1.2rem; font-weight: 600; }
.btn-back { background: rgba(255,255,255,0.15); border: 1px solid rgba(255,255,255,0.3); color: white; padding: 0.6rem 1rem; border-radius: 8px; cursor: pointer; font-weight: 600; transition: 0.3s; }
.btn-back:hover { background: rgba(255,255,255,0.25); border-color: white; }
.form-content { padding: 1.5rem; max-width: 700px; margin: 0 auto; }
.form-section { background: white; padding: 1.5rem; border-radius: 12px; box-shadow: 0 4px 15px rgba(0,0,0,0.03); margin-bottom: 2rem; border: 1px solid #f1f5f9; }
.form-section h3 { margin: 0 0 1.2rem 0; color: #166534; font-size: 1.1rem; border-bottom: 2px solid #f1f5f9; padding-bottom: 0.5rem; }
.form-group { margin-bottom: 1.2rem; display: flex; flex-direction: column; width: 100%; box-sizing: border-box; }
.form-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 1rem; width: 100%; box-sizing: border-box; }
label { font-weight: 600; margin-bottom: 0.5rem; font-size: 0.85rem; color: #475569; text-transform: uppercase; letter-spacing: 0.5px; }
input, select, textarea { width: 100%; box-sizing: border-box; border: 1.5px solid #cbd5e1; border-radius: 8px; padding: 0.8rem; font-size: 1rem; color: #1e293b; background: #f8fafc; transition: 0.2s; }
input:focus, select:focus, textarea:focus { outline: none; border-color: #22c55e; background: white; box-shadow: 0 0 0 3px rgba(34, 197, 94, 0.1); }
.custom-file-upload { border: 2px dashed #cbd5e1; border-radius: 8px; padding: 2rem; text-align: center; background: #f8fafc; }
.custom-file-upload input { display: none; }
.file-label { color: #166534; font-weight: bold; cursor: pointer; font-size: 1.1rem; padding: 0.5rem; display: block; }
.file-count { margin-top: 10px; color: #d97706; font-weight: bold; }
.btn-submit { width: 100%; padding: 1.2rem; background: #166534; color: white; border: none; border-radius: 8px; font-weight: 700; font-size: 1.1rem; cursor: pointer; transition: 0.2s; box-shadow: 0 4px 6px rgba(22, 101, 52, 0.2); }
.btn-submit:hover:not(:disabled) { transform: translateY(-2px); background: #14532d; box-shadow: 0 6px 12px rgba(22, 101, 52, 0.3); }
.btn-submit:disabled { background: #94a3b8; cursor: not-allowed; transform: none; box-shadow: none; }
.progress-container { margin-bottom: 1.5rem; background: #e2e8f0; border-radius: 8px; overflow: hidden; position: relative; height: 30px; box-shadow: inset 0 2px 4px rgba(0,0,0,0.1); }
.progress-bar { height: 100%; background: linear-gradient(90deg, #22c55e, #16a34a); transition: width 0.3s ease; }
.progress-text { position: absolute; width: 100%; text-align: center; top: 50%; left: 0; transform: translateY(-50%); font-weight: bold; color: #1e293b; font-size: 0.9rem; z-index: 10; text-shadow: 0px 0px 2px white; }
@media (max-width: 600px) { .form-grid { grid-template-columns: 1fr; } .form-content { padding: 1rem; } }
</style>