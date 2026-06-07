import axios from 'axios';

// ==========================================
// SERVIÇO DE API (PAINEL DE ADMINISTRAÇÃO)
// ==========================================
// Como o painel lida com rotas protegidas no Rust, precisamos garantir que
// toda requisição leve a "pulseirinha VIP" (o token JWT).

const api = axios.create({
    baseURL: import.meta.env.VITE_API_URL || (import.meta.env.DEV ? 'http://localhost:8000/api' : '/api'),
});

// INTERCEPTOR DE REQUISIÇÃO (REQUEST)
// Isso age como um "pedágio". Toda vez que o Axios for fazer um pedido pro servidor,
// ele passa por aqui primeiro.
api.interceptors.request.use((config) => {
    // Verifica se estamos enviando fotos (FormData). Se sim, o Axios descobre o 
    // Content-Type sozinho. Se não, forçamos 'application/json'.
    // *Pitfall*: Se você forçar application/json num upload de imagem, o backend quebra!
    if (config.data instanceof FormData) {
        delete config.headers['Content-Type'];
    } else {
        config.headers['Content-Type'] = 'application/json';
    }

    // Busca o Token JWT salvo no LocalStorage e anexa no cabeçalho 'Authorization'.
    // É assim que o Axum no backend sabe quem somos!
    const token = localStorage.getItem('authToken');
    if (token) {
        config.headers.Authorization = `Bearer ${token}`;
    }

    return config;
});

export default api;

// INTERCEPTOR DE RESPOSTA (RESPONSE)
// Outro "pedágio", mas para a volta! Toda vez que o backend responde, passa por aqui.
api.interceptors.response.use(
    (response) => response, // Se deu sucesso (HTTP 2xx), passa direto.
    (error) => {
        // Se deu erro 401 (Unauthorized), significa que nosso Token JWT expirou 
        // ou é inválido. A ação correta é deslogar o usuário e jogá-lo pra tela de login.
        if (error.response && error.response.status === 401) {
            localStorage.removeItem("authToken");
            if (window.location.pathname !== "/") {
                window.location.href = "/";
            }
        }
        return Promise.reject(error);
    }
);
