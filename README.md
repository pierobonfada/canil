# Documentação Oficial - Sistema de Adoção e Gestão de Animais

## 1. Visão Geral do Sistema
O **Canil System** é uma plataforma moderna, dividida em três pilares principais:
1. **API Backend**: Desenvolvida em Rust (framework Axum) com banco de dados SQLite para ultra-performance e confiabilidade.
2. **Site Público**: Portal em Vue.js para potenciais adotantes pesquisarem e conhecerem os animais.
3. **Painel de Administração**: Portal seguro em Vue.js destinado à equipe do abrigo, cuidadores e gestores.

---

## 2. Portal Público (Visão do Adotante)
O site acessível ao público foi projetado para facilitar a triagem e o "match" entre adotantes e animais. 

### 2.1 Pesquisa e Filtros
O portal dispõe de um sistema avançado de filtros responsivos.
* **Nome ou Espécie**: Busca direta de texto.
* **Cor Predominante**: Filtragem por cor de pelagem.
* **Porte**: Pequeno, Médio ou Grande.
* **Comportamentos**: Tolerância/Comportamento com Cães, Gatos e Humanos (Opções: Sociável, Tolerante, Não Sociável, Desconhecido).
* **Fase da Vida (Idade)**: O backend abstrai as idades automaticamente baseado no ano de nascimento, permitindo filtros de "Filhotes (0-1 ano)", "Adultos Jovens (2-3 anos)", "Adultos (4-7 anos)" e "Idosos (8+ anos)".

### 2.2 Fichas dos Animais
Cada animal possui um perfil que abrange:
* Galeria de fotos (com indicação da imagem de capa).
* Dados comportamentais e grau de independência.
* Condições médicas (vacinado, vermifugado e lista dinâmica de doenças/condições especiais).
* Histórico textual ou descrição da personalidade.

### 2.3 Solução de Contato (Interesse)
Ao clicar no botão para adotar ou falar sobre um animal, o portal exibe apenas os **tutores vinculados àquele animal específico**, impedindo que telefones da administração geral sejam inundados com mensagens de animais que eles não cuidam diretamente.

---

## 3. Painel Administrativo (Gestão e Cuidadores)

O painel fornece total controle gerencial do ecossistema, incluindo logs, proteção de contas e dados consolidados.

### 3.1 Níveis de Acesso e Proteções
* **Master (Gestor Geral)**:
  * Cria, edita e deleta contas de outros administradores.
  * Pode bloquear manualmente contas suspeitas.
  * Tem acesso exclusivo ao terminal de **Logs de Segurança** do sistema.
* **Admin (Cuidador/Voluntário)**:
  * Gerencia os cadastros de animais (fotos, condições médicas, dados).
  * Vê o painel de estatísticas, mas não acessa a aba de usuários e configurações de segurança da equipe.

**Proteções e Mecanismos de Autenticação**:
* **Login Criptografado**: Todas as senhas utilizam salt hash pelo algoritmo robusto `bcrypt`.
* **JWT Tokens**: A sessão é validada via JSON Web Tokens nas rotas privadas.
* **Troca de Senha Mandatória**: Usuários recém-criados possuem a flag `is_first_login = 1` no banco. Eles recebem uma senha provisória da direção e, ao entrarem, o sistema intercepta suas ações e exige a troca para uma senha pessoal inviolável imediatamente.
* **Bloqueio Automático Anti-Intrusão**: Se um usuário errar a senha muitas vezes (`failed_attempts`), a conta é travada (`is_locked = 1`) para impedir ataques de Força Bruta. Um Master deve atuar para destravá-la.

### 3.2 Gerenciamento de Animais
A edição e criação de fichas de animais abrange todas as características mencionadas no site e mais:
* **Vinculação de Tutores**: Um voluntário pode "Assumir a Tutoria" de um animal com o simples apertar de um botão. Isso exibe publicamente o seu nome e seu WhatsApp/Email na ficha daquele bichinho como ponto de contato. Um animal pode ter múltiplos tutores ao mesmo tempo.
* **Ocultação**: Permite ocultar um animal (`is_active = 0`) para adoções encaminhadas ou tratamentos sigilosos sem deletá-lo do banco.

### 3.3 Tratamento Inteligente de Imagens
Não há risco de servidores lotados com arquivos de celulares pesando 20MB.
* **Redimensionamento Automático no Servidor**: O backend em Rust intercepta toda e qualquer imagem inserida (independente se PNG, Webp, etc.).
* O motor aplica o algoritmo `Lanczos3` (altíssima preservação de qualidade geométrica) e "encolhe" as resoluções de forma assintótica para não ultrapassar limites de tela (ex: ~800x800).
* A imagem é higienizada e convertida para um JPEG otimizado universal com nome único UUID (`a42f...411e.jpg`).

---

## 4. Analytics, Estatísticas e Rastreamento

O site monitora de forma "passiva e ética" o fluxo de dados para ajudar o abrigo a entender o mercado.

### 4.1 Dashboard Visão Geral
Disponibiliza os dados agregados para resposta rápida:
* Visitantes diários, semanais, mensais e anuais.
* Ranking Dinâmico: **Top Animais com Mais Visualizações** vs **Top Animais que Geraram Mais Contatos (Interesse real)**.
* **Interesses de Busca**: Identifica o que as pessoas mais pesquisam com base nas requisições do site público: mostra rankings em tempo real das "Cores", "Tamanhos", "Espécies" e "Fases de Vida (Idades)" mais requisitadas pelos adotantes.

### 4.2 Monitoramento de Sessões e Jornadas
Permite ao abrigo investigar o tráfego a fundo.
* **Rolagem Infinita e Listagem**: Acompanhe todos os visitantes distintos (IPs anonimizados se necessário, ou exibidos para uso interno).
* **Logs Temporais**: Registra cada clique. Qual animal o usuário abriu primeiro, para onde navegou, e em que animal apertou "contato". Mostra os payloads de dados em tempo real.
* **Filtros Avançados**: A auditoria cruza dados complexos. É possível filtrar as sessões pelo IP, data exata, ou simplesmente digitar o nome de um animal (ex: `Rex Bolinha`) e o sistema retornará todas as pessoas no mundo que abriram a página deste animal específico para você analisar seus comportamentos prévios.

---

## 5. Auditoria de Dados (Security Logs e Action Logs)
Para controle de um fluxo contínuo de vários funcionários/voluntários mexendo na base, a API alimenta duas rotinas vitais:
1. **Action Logs**: Qualquer modificação que um cuidador faz (editar o pelo de um cachorro, adotar/deletar um animal) é registrada. O banco guarda o ID de quem mudou, que horas, e o que mudou (com status `INFO` ou `WARNING`).
2. **Security Logs**: Acionados quando alguém tentar quebrar regras (tentativas falhas de invasão aos endpoints Master, endpoints sem autenticação, ou uso de tokens expirados). Marcados com status `CRITICAL`.
