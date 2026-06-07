# ==========================================
# STAGE 1: Build do Frontend (Site Público)
# ==========================================
FROM node:22-alpine AS builder-site
WORKDIR /app/site
COPY site/package*.json ./
RUN npm ci
COPY site/ ./
RUN npm run build

# ==========================================
# STAGE 2: Build do Frontend (Painel Admin)
# ==========================================
FROM node:22-alpine AS builder-admin
WORKDIR /app/admin-canil
COPY admin-canil/package*.json ./
RUN npm ci
COPY admin-canil/ ./
RUN npm run build

# ==========================================
# STAGE 3: Build do Backend (Rust)
# ==========================================
FROM rust:1-bookworm AS builder-rust
RUN apt-get update && apt-get install -y sqlite3
WORKDIR /app
COPY api-canil/ ./api-canil
WORKDIR /app/api-canil
# Cria o banco vazio e as tabelas com o schema antes de compilar
RUN sqlite3 canil.db < schema.sql
# Define a URL do banco durante o build para as macros do SQLx compilarem corretamente
ENV DATABASE_URL="sqlite:///app/api-canil/canil.db"
# Compila o projeto em modo Release (otimizado para produção)
RUN cargo build --release

# ==========================================
# STAGE 4: Imagem Final de Execução (Runtime)
# ==========================================
FROM debian:bookworm-slim
WORKDIR /app

# Instala dependências do sistema para o banco de dados e TLS
RUN apt-get update && apt-get install -y libsqlite3-0 ca-certificates && rm -rf /var/lib/apt/lists/*

# Copia as pastas de frontend geradas nos estágios 1 e 2
COPY --from=builder-site /app/site/dist ./site/dist
COPY --from=builder-admin /app/admin-canil/dist ./admin-canil/dist

# Copia o binário compilado no estágio 3
COPY --from=builder-rust /app/api-canil/target/release/api-canil ./api-canil/api-canil-bin

# Expor a porta 8000
EXPOSE 8000

# Variáveis de Ambiente default para o Render
ENV PORT=8000
ENV DATABASE_URL="sqlite:///app/api-canil/canil.db"
ENV SITE_DIST="/app/site/dist"
ENV ADMIN_DIST="/app/admin-canil/dist"

# Muda o diretório de trabalho para bater com os caminhos relativos do Rust (../site e ../admin-canil)
WORKDIR /app/api-canil

# Comando para rodar a API
CMD ["./api-canil-bin"]
