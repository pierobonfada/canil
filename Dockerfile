# ==========================================
# STAGE 1: Build do Frontend (Site Público)
# ==========================================
FROM node:18-alpine AS builder-site
WORKDIR /app/site
COPY site/package*.json ./
RUN npm ci
COPY site/ ./
RUN npm run build

# ==========================================
# STAGE 2: Build do Frontend (Painel Admin)
# ==========================================
FROM node:18-alpine AS builder-admin
WORKDIR /app/admin-canil
COPY admin-canil/package*.json ./
RUN npm ci
COPY admin-canil/ ./
RUN npm run build

# ==========================================
# STAGE 3: Build do Backend (Rust)
# ==========================================
FROM rust:1.75-bookworm AS builder-rust
WORKDIR /app
COPY api-canil/ ./api-canil
WORKDIR /app/api-canil
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
COPY --from=builder-rust /app/api-canil/target/release/api-canil ./api-canil-bin

# Expor a porta 8000
EXPOSE 8000

# Variáveis de Ambiente default para o Render
ENV PORT=8000
ENV DATABASE_URL="sqlite:///app/canil.db"

# Comando para rodar a API
CMD ["./api-canil-bin"]
