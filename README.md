# Sistema de Adoção e Gestão de Animais - Projeto de Estudo

Este repositório contém um sistema de adoção de animais projetado para ser uma ferramenta de aprendizado em desenvolvimento web moderno. O projeto engloba um backend construído em **Rust** e dois frontends (um site público e um painel de administração) desenvolvidos em **Vue.js** com **TypeScript**.

O foco deste projeto é servir como material de estudo didático para estudantes de programação, demonstrando como diferentes tecnologias interagem em uma aplicação completa.

## 1. Arquitetura do Sistema

O projeto está dividido em três pastas principais, cada uma responsável por uma camada do sistema:

1. **`api-canil/` (Backend em Rust):** 
   - Utiliza o framework **Axum** para gerenciamento de rotas e requisições HTTP.
   - Emprega **SQLx** para comunicação com um banco de dados SQLite (`canil.db`).
   - Implementa autenticação JWT e criptografia de senhas (bcrypt).
   - Manipula upload de imagens e processamento.

2. **`site/` (Frontend Público em Vue.js):**
   - Construído com **Vue 3 (Composition API)** e **TypeScript**.
   - Interface voltada para os adotantes buscarem e visualizarem os animais.
   - Usa o Vue Router para navegação e consumo da API pública do backend.

3. **`admin-canil/` (Painel Administrativo em Vue.js):**
   - Sistema seguro para funcionários do abrigo gerenciarem animais, usuários e logs.
   - Demonstra como lidar com rotas protegidas (Requires Auth), interceptação de requisições e persistência de sessão (JWT).

---

## 2. Guia de Estudos (Por Onde Começar)

Para aproveitar este projeto para aprendizado, siga este roteiro de leitura do código-fonte. O código está enriquecido com comentários explicando funções, propósitos e *pitfalls* (armadilhas) comuns.

### Passo 1: Entendendo o Backend (Rust)
Comece pela pasta `api-canil/src`:
- **`main.rs`**: Ponto de entrada do backend. Aprenda como o Axum é inicializado, como as rotas são agrupadas e como a conexão com o banco (Connection Pool) é injetada no estado global.
- **`models.rs`**: Onde as estruturas de dados (structs) são definidas. Entenda como as macros do `serde` (`Serialize`, `Deserialize`) transformam dados do banco em JSON para a web.
- **`handlers.rs` e `admin_handlers.rs`**: Onde a mágica acontece. Estude como as funções extraem dados da requisição (Query, Path, JSON) e retornam respostas tipadas. Veja como o middleware de autenticação bloqueia rotas administrativas.

### Passo 2: O Portal Público (Vue.js)
Vá para a pasta `site/src`:
- **`services/api.ts`**: Veja como o `fetch` ou `axios` é encapsulado em serviços tipados pelo TypeScript. Entender isso é vital para conectar Front e Back de forma segura.
- **`views/HomeView.vue`**: Uma excelente introdução à *Composition API* (`<script setup>`). Observe como a reatividade (`ref`, `reactive`) gerencia o estado dos filtros e listagens de animais, e como dados são buscados no evento `onMounted`.
- **`router/index.ts`**: Aprenda como criar um sistema de navegação Single Page Application (SPA).

### Passo 3: O Painel Administrativo (Autenticação e Segurança)
Por fim, abra `admin-canil/src`:
- **`views/DashboardView.vue`**: Entenda a renderização de componentes com base no estado de carregamento e dados obtidos de endpoints protegidos.
- Estude os conceitos de Guardas de Rota (*Route Guards*) no router para bloquear páginas de quem não está logado, e o fluxo de login/troca de senha obrigatória.

---

## 3. Conceitos Abordados

Ao navegar por este código, você aprenderá:
- **Rust:** Segurança de memória, *pattern matching*, *borrow checker*, *lifetimes* (em contextos assíncronos) e serialização com Serde.
- **TypeScript:** Tipagem estática no frontend, interfaces, e uso seguro de retornos de API.
- **Vue.js:** Reatividade, *Lifecycle hooks*, *prop drilling*, componentes, diretivas (`v-if`, `v-for`) e gerenciamento de estado.
- **Arquitetura Web:** REST APIs, CORS, tokens JWT, hash de senhas, injeção de dependência e sanitização de dados.
