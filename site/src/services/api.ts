import axios from 'axios';

// ==========================================
// SERVIÇO DE API (FRONTEND PÚBLICO)
// ==========================================
// Aqui configuramos o Axios, uma biblioteca HTTP para fazer requisições ao Backend (Rust).
// Usar uma instância ('axios.create') é melhor que usar o axios global, pois
// permite definir uma URL base (baseURL) comum para todas as chamadas.

const api = axios.create({
  // Tenta pegar a URL do arquivo .env. Se não existir, tenta inferir dinamicamente 
  // usando o hostname atual na porta 8000.
  baseURL: import.meta.env.VITE_API_URL || `http://${window.location.hostname}:8000/api`,
  timeout: 10000, // Se o backend demorar mais de 10s, a requisição falha (evita tela infinita de loading).
});

export default api;
