<template>
  <div class="home-view">
    <!-- Hero Section -->
    <header class="hero">
      <div class="hero-content">
        <div class="paw-icon">🐾</div>
        <h1>Encontre seu<br/><span class="highlight">Novo Melhor Amigo</span></h1>
        <p class="subtitle">Adotar é encher a casa de alegria e o coração de amor. <br/>Dê uma chance para um focinho carente!</p>
        <div class="hero-buttons">
          <button class="btn-primary" @click="scrollToAnimals">
            Ver Peludinhos ⬇️
          </button>
          <button class="btn-outline hero-btn" @click="scrollToFilters">
            Filtros 🔍
          </button>
        </div>
      </div>
      <div class="hero-bg-shapes">
        <div class="shape shape-1"></div>
        <div class="shape shape-2"></div>
      </div>
    </header>

    <!-- Filters Section -->
    <section class="filters-section glass-panel" id="filters">
      <h2 class="section-title">🔍 Como é o amigo que você procura?</h2>
      <div class="filters-grid">
        <div class="filter-group">
          <label>Nome do Animal</label>
          <input type="text" v-model="filters.name" @input="onNameInput" placeholder="Busque por Rex, Mingau..." class="filter-input" />
        </div>
        <div class="filter-group">
          <label>Espécie</label>
          <select v-model="filters.species" @change="resetAndFetch">
            <option value="">Qualquer um</option>
            <option value="Cachorro">🐶 Cachorro</option>
            <option value="Gato">🐱 Gato</option>
          </select>
        </div>
        <div class="filter-group">
          <label>Cor Predominante</label>
          <select v-model="filters.predominant_color" @change="resetAndFetch">
            <option value="">Qualquer cor</option>
            <option value="Branco">Branco</option>
            <option value="Preto">Preto</option>
            <option value="Caramelo">Caramelo (O clássico!)</option>
            <option value="Cinza">Cinza</option>
            <option value="Marrom">Marrom</option>
            <option value="Tricolor">Tricolor</option>
            <option value="Bicolor">Bicolor</option>
            <option value="Laranja">Laranja (Garfield vibe)</option>
            <option value="Outra">Outra</option>
          </select>
        </div>
        <div class="filter-group">
          <label>Porte</label>
          <select v-model="filters.size" @change="resetAndFetch">
            <option value="">Qualquer tamanho</option>
            <option value="Pequeno">Pequeno (Cabe no colo)</option>
            <option value="Médio">Médio</option>
            <option value="Grande">Grande (Ursinho)</option>
          </select>
        </div>
        <div class="filter-group">
          <label>Idade (Fase da Vida)</label>
          <select v-model="filters.age_category" @change="resetAndFetch">
            <option value="">Qualquer idade</option>
            <option value="Filhote">Filhote (0 a 1 ano)</option>
            <option value="Adulto Jovem">Adulto Jovem (2 a 3 anos)</option>
            <option value="Adulto">Adulto (4 a 7 anos)</option>
            <option value="Idoso">Idoso (8+ anos)</option>
          </select>
        </div>
        <div class="filter-group">
          <label>Comportamento com Cães</label>
          <select v-model="filters.behavior_dogs" @change="resetAndFetch">
            <option value="">Qualquer um</option>
            <option value="Dócil">Dócil</option>
            <option value="Neutro">Neutro</option>
            <option value="Agressivo">Agressivo</option>
            <option value="Desconhecido">Desconhecido</option>
          </select>
        </div>
        <div class="filter-group">
          <label>Comportamento com Gatos</label>
          <select v-model="filters.behavior_cats" @change="resetAndFetch">
            <option value="">Qualquer um</option>
            <option value="Dócil">Dócil</option>
            <option value="Neutro">Neutro</option>
            <option value="Agressivo">Agressivo</option>
            <option value="Desconhecido">Desconhecido</option>
          </select>
        </div>
        <div class="filter-group">
          <label>Comportamento com Humanos</label>
          <select v-model="filters.behavior_humans" @change="resetAndFetch">
            <option value="">Qualquer um</option>
            <option value="Dócil">Dócil</option>
            <option value="Medroso">Medroso</option>
            <option value="Agressivo">Agressivo</option>
            <option value="Desconhecido">Desconhecido</option>
          </select>
        </div>
      </div>
    </section>

    <!-- Animals Grid -->
    <main class="animals-container">
      <div v-if="animals.length === 0 && !isLoading" class="empty-state">
        <div class="empty-icon">😿</div>
        <h3>Oops! Pisaram na minha pata!</h3>
        <p>Não encontramos nenhum amiguinho com essas características exatas. Que tal mudar um pouco os filtros?</p>
        <button class="btn-outline" @click="clearFilters">Limpar Filtros</button>
      </div>

      <div class="animals-grid">
        <div 
          v-for="animal in animals" 
          :key="animal.id" 
          class="animal-card glass-panel"
          @click="$router.push(`/animal/${animal.id}`)"
        >
          <div class="card-image-wrapper">
            <img :src="getPrimaryPhoto(animal)" :alt="animal.name" loading="lazy" />
            <div class="card-badges">
              <span class="badge species">{{ animal.species === 'Cachorro' ? '🐶' : '🐱' }}</span>
              <span v-if="isPuppyOrKitten(animal.birth_year)" class="badge tag-new">Bebê</span>
            </div>
          </div>
          <div class="card-content">
            <div class="card-header">
              <h3>{{ animal.name }}</h3>
              <span class="age">{{ calculateAge(animal.birth_year) }}</span>
            </div>
            <p class="breed">{{ animal.breed || 'SRD - Sem Raça Definida' }}</p>
            <div class="traits">
              <span class="trait">{{ animal.size }}</span>
              <span class="trait">{{ animal.predominant_color || animal.coat_color }}</span>
            </div>
            <p class="description-preview">{{ truncate(animal.description, 80) }}</p>
          </div>
        </div>
      </div>

      <!-- Infinite Scroll Trigger -->
      <div ref="loadMoreTrigger" class="loading-trigger">
        <div v-if="isLoading" class="loader">
          <div class="paw-loader">🐾</div>
          <p>Farejando mais amiguinhos...</p>
        </div>
        <div v-else-if="hasMore" class="scroll-prompt">
          <p>Role para baixo para ver mais</p>
        </div>
        <div v-else-if="animals.length > 0" class="end-message">
          <p>Você chegou ao fim da fila! Todos estão ansiosos por um lar. ❤️</p>
        </div>
      </div>
    </main>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted, onUnmounted } from 'vue';
import api from '../services/api';

const animals = ref<any[]>([]);
const isLoading = ref(false);
const hasMore = ref(true);
const page = ref(1);
const limit = 12;
const loadMoreTrigger = ref<HTMLElement | null>(null);

const filters = reactive({
  name: '',
  species: '',
  predominant_color: '',
  size: '',
  behavior_dogs: '',
  behavior_cats: '',
  behavior_humans: '',
  age_category: ''
});

const getPrimaryPhoto = (animal: any) => {
  if (!animal.photos || animal.photos.length === 0) return 'https://via.placeholder.com/400x400?text=Sem+Foto';
  const primary = animal.photos.find((p: any) => p.is_primary);
  const path = primary ? primary.file_path : animal.photos[0].file_path;
  const baseUrl = (import.meta.env.VITE_API_URL || `http://${window.location.hostname}:8000/api`).replace('/api', '');
  return `${baseUrl}/${path.startsWith('uploads') ? path : 'uploads/' + path}`;
};

const calculateAge = (birthYear: number) => {
  const currentYear = new Date().getFullYear();
  const age = currentYear - birthYear;
  if (age === 0) return 'Menos de 1 ano';
  if (age === 1) return '1 ano';
  return `${age} anos`;
};

const isPuppyOrKitten = (birthYear: number) => {
  return (new Date().getFullYear() - birthYear) <= 1;
};

const truncate = (text: string, length: number) => {
  if (!text) return '';
  return text.length > length ? text.substring(0, length) + '...' : text;
};

const fetchAnimals = async () => {
  if (isLoading.value || !hasMore.value) return;
  isLoading.value = true;
  
  try {
    const params: any = {
      page: page.value,
      limit,
      ...filters
    };
    
    // Map age_category to min/max age
    if (params.age_category) {
      if (params.age_category === 'Filhote') {
        params.age_min = 0; params.age_max = 1;
      } else if (params.age_category === 'Adulto Jovem') {
        params.age_min = 2; params.age_max = 3;
      } else if (params.age_category === 'Adulto') {
        params.age_min = 4; params.age_max = 7;
      } else if (params.age_category === 'Idoso') {
        params.age_min = 8; params.age_max = 30;
      }
    }

    // Remove empty filters
    Object.keys(params).forEach(key => {
      if (params[key] === '') delete params[key];
    });

    // Track search event if there are active filters (excluding page and limit)
    if (Object.keys(params).length > 2 && page.value === 1) {
      let visitorId = localStorage.getItem('visitor_id');
      if (visitorId) {
        const payload = { ...params };
        delete payload.page;
        delete payload.limit;
        api.post('/public/analytics', {
          visitor_id: visitorId,
          event_type: 'search',
          path: '/public/animais',
          animal_id: null,
          payload
        }).catch(() => {});
      }
    }

    const res = await api.get('/public/animais', { params });
    const data = res.data;
    
    if (data.length < limit) {
      hasMore.value = false;
    }
    
    animals.value = [...animals.value, ...data];
    page.value++;
  } catch (error) {
    console.error("Erro ao buscar animais:", error);
  } finally {
    isLoading.value = false;
  }
};

const resetAndFetch = () => {
  animals.value = [];
  page.value = 1;
  hasMore.value = true;
  fetchAnimals();
};

let searchTimeout: any;
const onNameInput = () => {
  clearTimeout(searchTimeout);
  searchTimeout = setTimeout(() => {
    resetAndFetch();
  }, 400);
};

const clearFilters = () => {
  filters.name = '';
  filters.species = '';
  filters.predominant_color = '';
  filters.size = '';
  filters.behavior_dogs = '';
  filters.behavior_cats = '';
  filters.behavior_humans = '';
  resetAndFetch();
};

const scrollToFilters = () => {
  document.getElementById('filters')?.scrollIntoView({ behavior: 'smooth' });
};

const scrollToAnimals = () => {
  document.querySelector('.animals-container')?.scrollIntoView({ behavior: 'smooth' });
};

// Intersection Observer for Infinite Scroll
let observer: IntersectionObserver;

onMounted(() => {
  observer = new IntersectionObserver((entries) => {
    if (entries[0].isIntersecting && hasMore.value && !isLoading.value) {
      fetchAnimals();
    }
  }, { rootMargin: '200px' });
  
  if (loadMoreTrigger.value) {
    observer.observe(loadMoreTrigger.value);
  }
  
  // Initial fetch
  fetchAnimals();
});

onUnmounted(() => {
  if (observer && loadMoreTrigger.value) {
    observer.unobserve(loadMoreTrigger.value);
  }
});
</script>

<style scoped>
.home-view {
  min-height: 100vh;
}

/* Hero Section */
.hero {
  position: relative;
  min-height: 70vh;
  display: flex;
  align-items: center;
  justify-content: center;
  text-align: center;
  overflow: hidden;
  padding: 2rem;
}

.hero-content {
  position: relative;
  z-index: 2;
  max-width: 800px;
}

.paw-icon {
  font-size: 4rem;
  margin-bottom: 1rem;
  animation: bounce 2s infinite;
  display: inline-block;
}

@keyframes bounce {
  0%, 20%, 50%, 80%, 100% { transform: translateY(0); }
  40% { transform: translateY(-20px); }
  60% { transform: translateY(-10px); }
}

.hero h1 {
  font-size: 4rem;
  line-height: 1.1;
  color: var(--text-dark);
  margin-bottom: 1.5rem;
  letter-spacing: -1px;
}

.highlight {
  color: var(--primary-color);
  position: relative;
}
.highlight::after {
  content: '';
  position: absolute;
  bottom: 5px;
  left: 0;
  width: 100%;
  height: 12px;
  background: var(--secondary-color);
  z-index: -1;
  opacity: 0.5;
  border-radius: 6px;
}

.subtitle {
  font-size: 1.3rem;
  color: var(--text-light);
  margin-bottom: 2.5rem;
  line-height: 1.6;
}

.btn-primary {
  background: var(--primary-color);
  color: white;
  border: none;
  padding: 1.2rem 2.5rem;
  font-size: 1.2rem;
  font-weight: 700;
  border-radius: 50px;
  box-shadow: 0 10px 20px rgba(79, 70, 229, 0.3);
  transition: all 0.3s cubic-bezier(0.25, 0.8, 0.25, 1);
}

.btn-primary:hover {
  transform: translateY(-5px) scale(1.02);
  box-shadow: 0 15px 25px rgba(79, 70, 229, 0.4);
  background: #4338ca;
}

.hero-buttons {
  display: flex;
  gap: 1rem;
  justify-content: center;
}

.hero-btn {
  border-width: 2px;
  background: rgba(255, 255, 255, 0.1);
  color: var(--primary-color);
}
.hero-btn:hover {
  background: var(--primary-color);
  color: white;
}

/* Background Shapes */
.hero-bg-shapes {
  position: absolute;
  top: 0; left: 0; width: 100%; height: 100%;
  z-index: 1;
  pointer-events: none;
}

.shape {
  position: absolute;
  border-radius: 50%;
  filter: blur(80px);
  opacity: 0.6;
}

.shape-1 {
  background: var(--secondary-color);
  width: 400px; height: 400px;
  top: -100px; left: -100px;
}

.shape-2 {
  background: var(--primary-color);
  width: 500px; height: 500px;
  bottom: -200px; right: -100px;
}

/* Filters */
.filters-section {
  max-width: 1200px;
  margin: -50px auto 3rem;
  padding: 2rem;
  position: relative;
  z-index: 10;
}

.section-title {
  margin-bottom: 1.5rem;
  font-size: 1.5rem;
  color: var(--primary-color);
}

.filters-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
  gap: 1.5rem;
}

.filter-group {
  display: flex;
  flex-direction: column;
}

.filter-group label {
  font-weight: 600;
  font-size: 0.9rem;
  margin-bottom: 0.5rem;
  color: var(--text-dark);
}

.filter-group select {
  padding: 1rem;
  border-radius: 12px;
  border: 1px solid rgba(0,0,0,0.1);
  background: white;
  font-family: inherit;
  font-size: 1rem;
  color: var(--text-dark);
  box-shadow: inset 0 2px 4px rgba(0,0,0,0.02);
  transition: all 0.2s;
  cursor: pointer;
}

.filter-group select:focus, .filter-input:focus {
  outline: none;
  border-color: var(--primary-color);
  box-shadow: 0 0 0 3px rgba(79, 70, 229, 0.2);
}

.filter-input {
  padding: 1rem;
  border-radius: 12px;
  border: 1px solid rgba(0,0,0,0.1);
  background: white;
  font-family: inherit;
  font-size: 1rem;
  color: var(--text-dark);
  box-shadow: inset 0 2px 4px rgba(0,0,0,0.02);
  transition: all 0.2s;
}

/* Animals Grid */
.animals-container {
  max-width: 1200px;
  margin: 0 auto;
  padding: 0 2rem 4rem;
}

.animals-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: 2rem;
}

.animal-card {
  cursor: pointer;
  overflow: hidden;
  transition: all 0.3s cubic-bezier(0.25, 0.8, 0.25, 1);
  display: flex;
  flex-direction: column;
  height: 100%;
}

.animal-card:hover {
  transform: translateY(-10px);
  box-shadow: 0 15px 35px rgba(0,0,0,0.1);
}

.card-image-wrapper {
  position: relative;
  height: 250px;
  overflow: hidden;
}

.card-image-wrapper img {
  width: 100%;
  height: 100%;
  object-fit: cover;
  transition: transform 0.5s;
}

.animal-card:hover .card-image-wrapper img {
  transform: scale(1.08);
}

.card-badges {
  position: absolute;
  top: 1rem;
  left: 1rem;
  display: flex;
  gap: 0.5rem;
}

.badge {
  background: white;
  padding: 0.4rem 0.8rem;
  border-radius: 20px;
  font-weight: 700;
  font-size: 0.9rem;
  box-shadow: 0 4px 10px rgba(0,0,0,0.1);
}

.badge.tag-new {
  background: var(--secondary-color);
  color: #fff;
}

.card-content {
  padding: 1.5rem;
  flex: 1;
  display: flex;
  flex-direction: column;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 0.5rem;
}

.card-header h3 {
  font-size: 1.5rem;
  color: var(--primary-color);
}

.age {
  font-weight: 600;
  color: var(--text-light);
  background: #f1f5f9;
  padding: 0.3rem 0.6rem;
  border-radius: 8px;
  font-size: 0.85rem;
}

.breed {
  color: var(--text-light);
  font-size: 0.9rem;
  margin: 0 0 1rem 0;
}

.traits {
  display: flex;
  gap: 0.5rem;
  margin-bottom: 1rem;
  flex-wrap: wrap;
}

.trait {
  background: rgba(79, 70, 229, 0.1);
  color: var(--primary-color);
  padding: 0.4rem 0.8rem;
  border-radius: 8px;
  font-size: 0.85rem;
  font-weight: 600;
}

.description-preview {
  color: var(--text-dark);
  font-size: 0.95rem;
  line-height: 1.5;
  margin-top: auto;
  opacity: 0.8;
}

/* Empty State */
.empty-state {
  text-align: center;
  padding: 4rem 2rem;
}

.empty-icon {
  font-size: 5rem;
  margin-bottom: 1rem;
}

.empty-state h3 {
  font-size: 2rem;
  color: var(--primary-color);
  margin-bottom: 1rem;
}

.empty-state p {
  font-size: 1.2rem;
  color: var(--text-light);
  margin-bottom: 2rem;
}

.btn-outline {
  background: transparent;
  border: 2px solid var(--primary-color);
  color: var(--primary-color);
  padding: 1rem 2rem;
  font-size: 1.1rem;
  font-weight: 700;
  border-radius: 50px;
  transition: 0.3s;
}

.btn-outline:hover {
  background: var(--primary-color);
  color: white;
}

/* Loading */
.loading-trigger {
  padding: 3rem 0;
  text-align: center;
}

.loader {
  display: flex;
  flex-direction: column;
  align-items: center;
  color: var(--primary-color);
  font-weight: 600;
}

.paw-loader {
  font-size: 2.5rem;
  animation: pulse 1.5s infinite alternate;
}

@keyframes pulse {
  0% { transform: scale(1); opacity: 0.5; }
  100% { transform: scale(1.3); opacity: 1; }
}

.scroll-prompt, .end-message {
  color: var(--text-light);
  font-weight: 500;
  opacity: 0.7;
}

@media (max-width: 768px) {
  .hero h1 { font-size: 2.8rem; }
  .filters-section { margin: 0 1rem 2rem; padding: 1.5rem; }
  .animals-container { padding: 0 1rem 2rem; }
}
</style>
