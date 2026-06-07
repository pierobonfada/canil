<!-- ========================================== -->
<!-- 🐾 PERFIL DO ANIMAL (A Página do "Match") -->
<!-- ========================================== -->
<!-- Quando o visitante clica num cachorro ou gato, é pra cá que ele vem!
Aqui mostramos o carrossel de fotos, a ficha médica completa, e o grande 
botão verde do WhatsApp pra ele entrar em contato com os tutores! -->
<template>
  <div class="animal-view">
    <div v-if="isLoading" class="loading-state">
      <div class="paw-loader">🐾</div>
      <p>Carregando detalhes do fofinho...</p>
    </div>
    
    <div v-else-if="error" class="error-state">
      <div class="empty-icon">😿</div>
      <h2>Ops!</h2>
      <p>{{ error }}</p>
      <button class="btn-primary" @click="$router.push('/')">Voltar para a Home</button>
    </div>

    <template v-else-if="animal">
      <nav class="top-nav">
        <button class="btn-back" @click="$router.push('/')">
          <span class="arrow">←</span> Voltar
        </button>
      </nav>

      <main class="animal-details-container">
        <div class="gallery-section">
          <div class="main-photo glass-panel">
            <img :src="getPhotoUrl(currentPhoto)" :alt="animal.name" />
          </div>
          <div class="thumbnails" v-if="animal.photos && animal.photos.length > 1">
            <div 
              v-for="photo in animal.photos" 
              :key="photo.file_path"
              class="thumbnail glass-panel"
              :class="{ active: currentPhoto === photo.file_path }"
              @click="currentPhoto = photo.file_path"
            >
              <img :src="getPhotoUrl(photo.file_path)" alt="Miniatura" />
            </div>
          </div>
        </div>

        <div class="info-section">
          <div class="header-info glass-panel">
            <div class="title-row">
              <h1>{{ animal.name }}</h1>
              <span class="badge species">{{ animal.species === 'Cachorro' ? '🐶' : '🐱' }}</span>
            </div>
            <p class="breed">{{ animal.breed || 'SRD (O famoso sem raça definida, único no mundo!)' }}</p>
            
            <div class="quick-facts">
              <div class="fact">
                <span class="fact-label">Idade</span>
                <span class="fact-value">{{ calculateAge(animal.birth_year) }}</span>
              </div>
              <div class="fact">
                <span class="fact-label">Porte</span>
                <span class="fact-value">{{ animal.size }}</span>
              </div>
              <div class="fact">
                <span class="fact-label">Cor</span>
                <span class="fact-value">{{ animal.predominant_color || animal.coat_color }}</span>
              </div>
              <div class="fact">
                <span class="fact-label">Pelagem</span>
                <span class="fact-value">{{ animal.coat_length }}</span>
              </div>
            </div>
          </div>

          <div class="description-card glass-panel">
            <h2>Sobre Mim</h2>
            <p>{{ animal.description }}</p>
          </div>

          <div class="details-grid">
            <div class="detail-card glass-panel">
              <h3>Comportamento</h3>
              <ul>
                <li><strong>Independência:</strong> {{ animal.independence }}</li>
                <li><strong>Com cães:</strong> {{ animal.behavior_dogs }}</li>
                <li><strong>Com gatos:</strong> {{ animal.behavior_cats }}</li>
                <li><strong>Com humanos:</strong> {{ animal.behavior_humans }}</li>
              </ul>
            </div>
            <div class="detail-card glass-panel">
              <h3>Saúde</h3>
              <ul>
                <li>{{ animal.is_vaccinated ? '✅ Vacinado' : '❌ Não vacinado' }}</li>
                <li>{{ animal.is_dewormed ? '✅ Vermifugado' : '❌ Não vermifugado' }}</li>
                <li v-if="animal.diseases && animal.diseases.length > 0">
                  <strong>Atenção a:</strong> {{ animal.diseases.join(', ') }}
                </li>
                <li v-else><strong>Condições Especiais:</strong> Nenhuma reportada</li>
              </ul>
            </div>
          </div>

          <div class="contact-section glass-panel">
            <h2>❤️ Quero Adotar!</h2>
            <p v-if="tutors.length > 0">Que incrível! Fale diretamente com o tutor(a) responsável por {{ animal.name }}:</p>
            <p v-else>Não há tutores cadastrados para este animal no momento.</p>

            <div class="tutors-list" v-if="tutors.length > 0">
              <div v-for="tutor in tutors" :key="tutor.email" class="tutor-card">
                <div class="tutor-info">
                  <strong>{{ tutor.name }}</strong>
                  <span>{{ formatPhone(tutor.phone) }}</span>
                </div>
                <div class="tutor-actions">
                  <a :href="getWhatsAppLink(tutor.phone, tutor.name)" @click="trackMessage('whatsapp', tutor.name)" target="_blank" class="btn-whatsapp">
                    📱 WhatsApp
                  </a>
                  <a :href="getEmailLink(tutor.email, tutor.name)" @click="trackMessage('email', tutor.name)" class="btn-email">
                    ✉️ E-mail
                  </a>
                </div>
              </div>
            </div>
          </div>

        </div>
      </main>
    </template>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useRoute } from 'vue-router';
import api from '../services/api';

const route = useRoute();
const animal = ref<any>(null);
const tutors = ref<any[]>([]);
const currentPhoto = ref('');
const isLoading = ref(true);
const error = ref('');

const fetchAnimal = async () => {
  try {
    const res = await api.get(`/public/animais/${route.params.id}`);
    animal.value = res.data.animal;
    tutors.value = res.data.tutors;
    
    if (animal.value.photos && animal.value.photos.length > 0) {
      const primary = animal.value.photos.find((p: any) => p.is_primary);
      currentPhoto.value = primary ? primary.file_path : animal.value.photos[0].file_path;
    }
  } catch (err: any) {
    if (err.response?.status === 404) {
      error.value = 'Amiguinho não encontrado. Talvez ele já tenha sido adotado!';
    } else {
      error.value = 'Ocorreu um erro ao carregar as informações.';
    }
  } finally {
    isLoading.value = false;
  }
};

const getPhotoUrl = (path: string) => {
  if (!path) return 'data:image/svg+xml;utf8,<svg xmlns="http://www.w3.org/2000/svg" width="400" height="400"><rect width="400" height="400" fill="%23cccccc"/><text x="50%" y="50%" fill="%23000000" text-anchor="middle" dy=".3em">Sem Foto</text></svg>';
  const baseUrl = (import.meta.env.VITE_API_URL || (import.meta.env.DEV ? 'http://localhost:8000/api' : '/api')).replace('/api', '');
  return `${baseUrl}/${path.startsWith('uploads') ? path : 'uploads/' + path}`;
};

const calculateAge = (birthYear: number) => {
  const currentYear = new Date().getFullYear();
  const age = currentYear - birthYear;
  if (age === 0) return 'Bebê (< 1 ano)';
  if (age === 1) return '1 ano';
  return `${age} anos`;
};

const formatPhone = (phone: string) => {
  const cleaned = ('' + phone).replace(/\D/g, '');
  if (cleaned.length === 11) {
    return `(${cleaned.substring(0, 2)}) ${cleaned.substring(2, 3)} ${cleaned.substring(3, 7)}-${cleaned.substring(7, 11)}`;
  } else if (cleaned.length === 10) {
    return `(${cleaned.substring(0, 2)}) ${cleaned.substring(2, 6)}-${cleaned.substring(6, 10)}`;
  }
  return phone;
};

const getWhatsAppLink = (phone: string, tutorName: string) => {
  const cleaned = ('' + phone).replace(/\D/g, '');
  const message = encodeURIComponent(`Olá ${tutorName}, vi o perfil do ${animal.value.name} no site de adoção e gostaria de mais informações! 🐶🐱`);
  return `https://wa.me/55${cleaned}?text=${message}`;
};

const getEmailLink = (email: string, tutorName: string) => {
  const subject = encodeURIComponent(`Interesse na adoção de: ${animal.value.name}`);
  const body = encodeURIComponent(`Olá ${tutorName},\n\nGostaria de mais informações sobre o processo de adoção do ${animal.value.name}.\n\nFico no aguardo.`);
  return `mailto:${email}?subject=${subject}&body=${body}`;
};

const trackMessage = (type: string, tutorName: string) => {
  const visitorId = localStorage.getItem('visitor_id');
  if (visitorId && animal.value) {
    api.post('/public/analytics', {
      visitor_id: visitorId,
      event_type: 'message_sent',
      path: route.fullPath,
      animal_id: animal.value.id,
      payload: { method: type, tutor: tutorName }
    }).catch(() => {});
  }
};

onMounted(() => {
  fetchAnimal();
  window.scrollTo(0, 0);
});
</script>

<style scoped>
.animal-view {
  min-height: 100vh;
  padding: 2rem;
  max-width: 1200px;
  margin: 0 auto;
}

.top-nav {
  margin-bottom: 2rem;
}

.btn-back {
  background: transparent;
  border: none;
  color: var(--primary-color);
  font-weight: 700;
  font-size: 1.1rem;
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.btn-back:hover .arrow {
  transform: translateX(-5px);
}

.arrow {
  transition: transform 0.3s;
}

.animal-details-container {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 3rem;
  align-items: start;
}

.main-photo {
  border-radius: 24px;
  overflow: hidden;
  aspect-ratio: 4/5;
  margin-bottom: 1rem;
}

.main-photo img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.thumbnails {
  display: flex;
  gap: 1rem;
  overflow-x: auto;
  padding-bottom: 0.5rem;
}

.thumbnail {
  width: 80px;
  height: 80px;
  border-radius: 12px;
  overflow: hidden;
  cursor: pointer;
  border: 3px solid transparent;
  flex-shrink: 0;
  transition: all 0.3s;
}

.thumbnail img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.thumbnail.active {
  border-color: var(--primary-color);
  transform: scale(1.05);
}

.info-section {
  display: flex;
  flex-direction: column;
  gap: 1.5rem;
}

.header-info {
  padding: 2rem;
}

.title-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 0.5rem;
}

.title-row h1 {
  font-size: 2.5rem;
  color: var(--primary-color);
}

.breed {
  color: var(--text-light);
  font-size: 1.2rem;
  margin-bottom: 2rem;
}

.quick-facts {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 1.5rem;
}

.fact {
  display: flex;
  flex-direction: column;
}

.fact-label {
  font-size: 0.9rem;
  color: var(--text-light);
  text-transform: uppercase;
  letter-spacing: 1px;
  margin-bottom: 0.3rem;
}

.fact-value {
  font-size: 1.1rem;
  font-weight: 600;
  color: var(--text-dark);
}

.description-card {
  padding: 2rem;
}

.description-card h2 {
  color: var(--secondary-color);
  margin-bottom: 1rem;
}

.description-card p {
  line-height: 1.8;
  font-size: 1.1rem;
}

.details-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 1.5rem;
}

.detail-card {
  padding: 1.5rem;
}

.detail-card h3 {
  color: var(--primary-color);
  margin-bottom: 1rem;
  font-size: 1.2rem;
}

.detail-card ul {
  list-style: none;
  padding: 0;
  margin: 0;
  display: flex;
  flex-direction: column;
  gap: 0.8rem;
}

.detail-card li {
  font-size: 1rem;
}

.contact-section {
  padding: 2rem;
  background: linear-gradient(135deg, rgba(79,70,229,0.1) 0%, rgba(6,182,212,0.1) 100%);
  border: 2px solid rgba(79,70,229,0.2);
}

.contact-section h2 {
  color: var(--primary-color);
  margin-bottom: 1rem;
}

.tutors-list {
  display: flex;
  flex-direction: column;
  gap: 1rem;
  margin-top: 1.5rem;
}

.tutor-card {
  background: white;
  padding: 1rem 1.5rem;
  border-radius: 12px;
  display: flex;
  justify-content: space-between;
  align-items: center;
  box-shadow: 0 4px 10px rgba(0,0,0,0.05);
}

.tutor-info {
  display: flex;
  flex-direction: column;
}

.tutor-info strong {
  font-size: 1.1rem;
  color: var(--text-dark);
}

.tutor-info span {
  color: var(--text-light);
  font-size: 0.9rem;
}

.tutor-actions {
  display: flex;
  gap: 0.5rem;
}

.btn-whatsapp {
  background: #25D366;
  color: white;
  padding: 0.6rem 1rem;
  border-radius: 8px;
  font-weight: 600;
  transition: 0.2s;
}

.btn-whatsapp:hover {
  background: #128C7E;
  transform: translateY(-2px);
}

.btn-email {
  background: #3b82f6;
  color: white;
  padding: 0.6rem 1rem;
  border-radius: 8px;
  font-weight: 600;
  transition: 0.2s;
}

.btn-email:hover {
  background: #2563eb;
  transform: translateY(-2px);
}

.loading-state, .error-state {
  height: 60vh;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  text-align: center;
}

.paw-loader {
  font-size: 4rem;
  animation: pulse 1s infinite alternate;
  margin-bottom: 1rem;
}

@keyframes pulse {
  0% { transform: scale(1); }
  100% { transform: scale(1.2); color: var(--primary-color); }
}

.error-state .empty-icon {
  font-size: 5rem;
  margin-bottom: 1rem;
}

.btn-primary {
  background: var(--primary-color);
  color: white;
  border: none;
  padding: 1rem 2rem;
  border-radius: 50px;
  font-weight: bold;
  margin-top: 1.5rem;
}

@media (max-width: 900px) {
  .animal-details-container {
    grid-template-columns: 1fr;
    gap: 2rem;
  }
  
  .details-grid {
    grid-template-columns: 1fr;
  }
  
  .tutor-card {
    flex-direction: column;
    align-items: flex-start;
    gap: 1rem;
  }
  
  .tutor-actions {
    width: 100%;
  }
  
  .btn-whatsapp, .btn-email {
    flex: 1;
    text-align: center;
  }
}
</style>
