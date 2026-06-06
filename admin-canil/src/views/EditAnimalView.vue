<template>
  <div class="edit-animal-container">
    <header class="top-bar">
      <button @click="$router.push('/dashboard')" class="btn-back">
        ← Voltar ao Painel
      </button>
      <h2>✏️ Editar Dados do Animal</h2>
    </header>

    <main class="form-content">
      <div v-if="isFetching" class="loading-state">
        Sincronizando dados...
      </div>

      <form v-else @submit.prevent="submitForm" class="animal-form">
        
        <section class="form-section">
          <h3>Atualizar Identificação</h3>
          <div class="form-group">
            <label>Nome do Animal *</label>
            <input type="text" v-model="form.name" required />
          </div>
          
          <div class="form-grid">
            <div class="form-group">
              <label>Espécie *</label>
              <select v-model="form.species" required>
                <option value="Cachorro">Cachorro</option>
                <option value="Gato">Gato</option>
              </select>
            </div>
            <div class="form-group">
              <label>Ano de Nasc. *</label>
              <input type="number" v-model="form.birth_year" required />
            </div>
          </div>
          
          <div class="form-group">
            <label>Raça</label>
            <input type="text" v-model="form.breed" placeholder="Deixe em branco se SRD" />
          </div>
        </section>

        <section class="form-section">
          <h3>Atualizar Perfil e Saúde</h3>
          <div class="form-grid">
            <div class="form-group">
              <label>Porte *</label>
              <select v-model="form.size" required>
                <option value="Pequeno">Pequeno</option>
                <option value="Médio">Médio</option>
                <option value="Grande">Grande</option>
              </select>
            </div>
            <div class="form-group">
              <label>Independência *</label>
              <select v-model="form.independence" required>
                <option value="Independente">Independente</option>
                <option value="Dependente">Dependente</option>
              </select>
            </div>
          </div>
          
          <div class="form-grid">
            <div class="form-group">
              <label>Com Cães *</label>
              <select v-model="form.behavior_dogs" required>
                <option value="Dócil">Dócil</option>
                <option value="Neutro">Neutro</option>
                <option value="Agressivo">Agressivo</option>
              </select>
            </div>
            <div class="form-group">
              <label>Com Humanos *</label>
              <select v-model="form.behavior_humans" required>
                <option value="Dócil">Dócil</option>
                <option value="Medroso">Medroso</option>
                <option value="Agressivo">Agressivo</option>
              </select>
            </div>
          </div>

          <div class="form-grid">
            <div class="form-group">
              <label>Vacinado? *</label>
              <select v-model="form.is_vaccinated" required>
                <option :value="true">Sim</option>
                <option :value="false">Não</option>
              </select>
            </div>
            <div class="form-group">
              <label>Vermifugado? *</label>
              <select v-model="form.is_dewormed" required>
                <option :value="true">Sim</option>
                <option :value="false">Não</option>
              </select>
            </div>
          </div>

          <div class="form-group">
            <label>Doenças / Observações</label>
            <input type="text" v-model="form.diseases" placeholder="Ex: Nenhuma" />
          </div>
        </section>

        <section class="form-section">
          <h3>Atualizar História e Fotos</h3>
          <div class="form-group">
            <label>Descrição Detalhada *</label>
            <textarea v-model="form.description" required rows="4"></textarea>
          </div>

          <div class="form-group" v-if="form.photos.length > 0">
            <label>Gerenciar Fotos Salvas (Clique na ⭐️ para definir a Capa)</label>
            <div class="current-photos-grid">
              <div 
                class="photo-item" 
                v-for="(photo, index) in form.photos" 
                :key="index"
                :class="{ 'photo-inactive': !photo.is_active, 'photo-primary': form.primary_photo === photo.file_path }"
              >
                <img :src="getPhotoUrl(photo.file_path)" alt="Foto salva" />
                
                <button 
                  type="button" 
                  class="btn-set-primary" 
                  title="Definir como Capa" 
                  @click="setPrimaryPhoto(photo.file_path)"
                  v-if="photo.is_active"
                >
                  ⭐
                </button>

                <button 
                  type="button" 
                  :class="photo.is_active ? 'btn-remove-photo' : 'btn-restore-photo'" 
                  @click="togglePhoto(index)" 
                  :title="photo.is_active ? 'Desativar foto' : 'Restaurar foto'"
                >
                  <span v-if="photo.is_active">&times;</span>
                  <span v-else>&#x21BA;</span>
                </button>
              </div>
            </div>
          </div>

          <div class="form-group">
            <label>Adicionar Novas Fotos</label>
            <div class="custom-file-upload warning-border">
              <input type="file" id="edit-upload" multiple accept="image/*" @change="handleFileUpload" />
              <label for="edit-upload" class="file-label text-blue">
                📷 Selecionar Novas Imagens
              </label>
              <p class="file-count" v-if="selectedFiles.length > 0">{{ selectedFiles.length }} arquivo(s) novo(s) pronto(s) para envio.</p>
            </div>
          </div>
        </section>

        <button type="submit" class="btn-submit" :disabled="isLoading">
          {{ isLoading ? 'Gravando Alterações...' : 'Salvar Alterações Permanentemente' }}
        </button>
      </form>
    </main>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import api from '../services/api'

const route = useRoute()
const router = useRouter()
const animalId = route.params.id

const isFetching = ref(true)
const isLoading = ref(false)
const selectedFiles = ref<File[]>([])

const form = ref({ 
  name: '', species: '', birth_year: null as any, breed: '', 
  is_vaccinated: '' as any, is_dewormed: '' as any, behavior_dogs: '', 
  behavior_humans: '', independence: '', size: '', 
  coat_color: 'Indefinida', coat_length: 'Curto', description: '', diseases: '',
  primary_photo: '',
  photos: [] as { file_path: string, is_active: boolean, is_primary: boolean }[]
})

const getPhotoUrl = (photo: string) => {
  const apiUrl = import.meta.env.VITE_API_URL || 'http://localhost:8000/api';
  const baseUrl = apiUrl.replace('/api', ''); 
  const cleanPhoto = photo.startsWith('uploads/') ? photo : `uploads/${photo}`;
  return `${baseUrl}/${cleanPhoto}`;
}

const handleFileUpload = (e: any) => {
  if (e.target.files) {
    selectedFiles.value = Array.from(e.target.files)
  }
}

const setPrimaryPhoto = (path: string) => {
  form.value.primary_photo = path;
}

const togglePhoto = (index: number) => {
  const photo = form.value.photos[index];
  if (!photo) return;
  photo.is_active = !photo.is_active;
  // Se inativarmos a foto que é capa, remove a seleção dela temporariamente
  if (!photo.is_active && form.value.primary_photo === photo.file_path) {
    form.value.primary_photo = '';
  }
}

onMounted(async () => {
  try {
    const res = await api.get(`/animais/${animalId}`)
    const data = res.data
    form.value = { 
      ...data, 
      diseases: data.diseases ? data.diseases.join(', ') : '',
      primary_photo: data.photos.find((p: any) => p.is_primary)?.file_path || '',
      photos: data.photos || []
    }
  } catch (e) {
    alert('Erro ao carregar dados.')
    router.push('/dashboard')
  } finally {
    isFetching.value = false
  }
})

const submitForm = async () => {
  isLoading.value = true
  const formData = new FormData()
  
  Object.entries(form.value).forEach(([key, value]) => {
    if (key !== 'photos' && value !== null && value !== '') {
      formData.append(key, value.toString())
    }
  })

  form.value.photos.forEach(photo => {
    if (photo.is_active) {
      formData.append('active_photos', photo.file_path)
    } else {
      formData.append('inactive_photos', photo.file_path)
    }
  })

  selectedFiles.value.forEach(file => {
    formData.append('photo', file)
  })

  try {
    await api.put(`/animais/${animalId}`, formData)
    alert('Animal atualizado com sucesso!')
    router.push('/dashboard')
  } catch (e: any) {
    alert('Erro ao salvar: ' + (e.response?.data?.error || 'Falha na conexão'))
  } finally {
    isLoading.value = false
  }
}
</script>

<style scoped>
.edit-animal-container { min-height: 100vh; background-color: #f8fafc; font-family: 'Segoe UI', Tahoma, sans-serif; padding-bottom: 3rem; }

.top-bar { 
  display: flex; align-items: center; background: linear-gradient(135deg, #1e3a8a 0%, #1e40af 100%); 
  padding: 1.2rem 1.5rem; color: white; position: sticky; top: 0; z-index: 100; 
  box-shadow: 0 4px 12px rgba(0,0,0,0.1); flex-wrap: nowrap; gap: 15px;
}
.top-bar h2 { margin: 0; font-size: 1.2rem; font-weight: 600; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
.btn-back { 
  flex-shrink: 0; white-space: nowrap; background: rgba(255,255,255,0.15); border: 1px solid rgba(255,255,255,0.3); 
  color: white; padding: 0.6rem 1rem; border-radius: 8px; cursor: pointer; font-weight: 600; transition: 0.3s; 
}
.btn-back:hover { background: rgba(255,255,255,0.25); border-color: white; }

.current-photos-grid { display: flex; flex-wrap: wrap; gap: 18px; margin-top: 10px; }
.photo-item { position: relative; width: 120px; height: 120px; border-radius: 12px; overflow: hidden; box-shadow: 0 4px 10px rgba(0,0,0,0.1); border: 3px solid #e2e8f0; transition: 0.3s; }
.photo-item img { width: 100%; height: 100%; object-fit: cover; }
.photo-inactive { filter: grayscale(100%); opacity: 0.5; border-color: #94a3b8; }
.photo-primary { border-color: #eab308; box-shadow: 0 0 12px rgba(234, 179, 8, 0.4); }

.btn-set-primary {
  position: absolute; top: 6px; left: 6px; background: white; border: none; border-radius: 50%;
  width: 26px; height: 26px; cursor: pointer; display: flex; align-items: center; justify-content: center;
  font-size: 14px; box-shadow: 0 2px 6px rgba(0,0,0,0.3); transition: 0.2s;
}
.btn-set-primary:hover { transform: scale(1.15); }

.btn-remove-photo { 
  position: absolute; top: 6px; right: 6px; background: #ef4444; color: white; border: none; 
  width: 26px; height: 26px; border-radius: 50%; cursor: pointer; display: flex; align-items: center; justify-content: center; 
  font-size: 20px; font-weight: bold; line-height: 1; box-shadow: 0 2px 6px rgba(0,0,0,0.3); transition: 0.2s; padding-bottom: 2px;
}
.btn-remove-photo:hover { background: #dc2626; transform: scale(1.15); }

.btn-restore-photo { 
  position: absolute; top: 6px; right: 6px; background: #22c55e; color: white; border: none; 
  width: 26px; height: 26px; border-radius: 50%; cursor: pointer; display: flex; align-items: center; justify-content: center; 
  font-size: 16px; font-weight: bold; line-height: 1; box-shadow: 0 2px 6px rgba(0,0,0,0.3); transition: 0.2s;
}
.btn-restore-photo:hover { background: #16a34a; transform: scale(1.15); }

.form-content { padding: 1.5rem; max-width: 700px; margin: 0 auto; }
.form-section { background: white; padding: 1.5rem; border-radius: 12px; box-shadow: 0 4px 15px rgba(0,0,0,0.03); margin-bottom: 2rem; border: 1px solid #f1f5f9; }
.form-section h3 { margin: 0 0 1.2rem 0; color: #1e40af; font-size: 1.1rem; border-bottom: 2px solid #f1f5f9; padding-bottom: 0.5rem; display: flex; align-items: center; gap: 8px;}

.form-group { margin-bottom: 1.2rem; display: flex; flex-direction: column; width: 100%; box-sizing: border-box; }
.form-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 1rem; width: 100%; box-sizing: border-box; }

label { font-weight: 600; margin-bottom: 0.5rem; font-size: 0.85rem; color: #475569; text-transform: uppercase; letter-spacing: 0.5px; }

input, select, textarea { width: 100%; box-sizing: border-box; border: 1.5px solid #cbd5e1; border-radius: 8px; padding: 0.8rem; font-size: 1rem; color: #1e293b; background: #f8fafc; transition: 0.2s; }
input:focus, select:focus, textarea:focus { outline: none; border-color: #3b82f6; background: white; box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.1); }

.custom-file-upload { border: 2px dashed #cbd5e1; border-radius: 8px; padding: 2rem; text-align: center; background: #f8fafc; transition: 0.2s; }
.warning-border { border-color: #d97706; background: #fffbeb; }
.custom-file-upload input { display: none; }
.file-label { font-weight: bold; cursor: pointer; font-size: 1.1rem; padding: 0.5rem; display: flex; flex-direction: column; gap: 8px;}
.text-blue { color: #1e40af; }
.file-count { margin-top: 10px; color: #1e40af; font-weight: bold; background: #eff6ff; padding: 5px; border-radius: 6px;}

.btn-submit { width: 100%; padding: 1.2rem; background: #3b82f6; color: white; border: none; border-radius: 8px; font-weight: 700; font-size: 1.1rem; cursor: pointer; transition: 0.2s; box-shadow: 0 4px 6px rgba(59, 130, 246, 0.2); display: flex; align-items: center; justify-content: center; gap: 8px;}
.btn-submit:hover:not(:disabled) { transform: translateY(-2px); background: #2563eb; box-shadow: 0 6px 12px rgba(59, 130, 246, 0.3); }
.btn-submit:disabled { background: #94a3b8; cursor: not-allowed; transform: none; box-shadow: none; }
.loading-state { text-align: center; padding: 3rem; color: #1e40af; font-weight: bold; font-size: 1.2rem; }

@media (max-width: 600px) {
  .form-grid { grid-template-columns: 1fr; }
  .form-content { padding: 1rem; }
}
</style>