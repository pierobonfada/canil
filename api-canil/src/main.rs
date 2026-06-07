// ==========================================
// PONTO DE ENTRADA: Configuração e inicialização do servidor Axum
// ==========================================
// Aqui importamos os módulos do nosso projeto. O Rust exige que declaremos 
// explicitamente os módulos que compõem o crate (pacote) usando 'pub mod':
pub mod auth;               // Funções para geração e validação de tokens JWT (segurança).
pub mod analytics_handlers; // Controladores (handlers) para rotas de estatísticas e métricas de acesso.
pub mod admin_handlers;     // Controladores para gestão de administradores (CRUD e rotas protegidas).
pub mod handlers;           // Controladores principais para gestão de animais e login.
pub mod image_utils;        // Utilitários para redimensionamento e salvamento assíncrono de imagens.
pub mod models;             // Estruturas de dados (structs) que representam as tabelas do banco e payloads.

use axum::{
    routing::{get, patch, post, put},
    Router,
};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use std::{env, net::SocketAddr, str::FromStr};
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::{ServeDir, ServeFile};

use crate::admin_handlers::{create_admin, get_admins, get_system_logs, update_admin_status, update_admin, delete_admin};
use crate::handlers::{
    change_password, create_animal, get_animal, get_animals, get_dashboard, login_handler,
    toggle_tutorship, update_animal, update_animal_status, update_preferences,
    get_public_animals, get_public_animal,
};
use crate::analytics_handlers::{
    register_event, get_dashboard_stats, get_sessions, get_session_details
};

#[tokio::main]
// FUNÇÃO PRINCIPAL
// A macro #[tokio::main] transforma a função main tradicional (síncrona) 
// em uma função assíncrona executada por um runtime (o Tokio). Isso é essencial
// no Rust para lidar com múltiplas requisições HTTP de forma concorrente sem bloquear a thread.
async fn main() {
    // Carrega variáveis do arquivo .env para o ambiente local.
    dotenvy::dotenv().ok();

    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL não configurada no .env");
    
    if env::var("JWT_SECRET").is_err() {
        panic!("ERRO: Variável de ambiente JWT_SECRET não encontrada no arquivo .env!");
    }

    // Conexão com o banco de dados SQLite.
    // Usamos o modo WAL (Write-Ahead Logging) que melhora significativamente a performance 
    // de leitura e escrita simultânea no SQLite.
    let connection_options = SqliteConnectOptions::from_str(&db_url)
        .expect("URL do banco inválida")
        .create_if_missing(true)
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal);

    // Cria um 'Pool' de conexões. O pool gerencia e reaproveita conexões ao banco, 
    // evitando a sobrecarga de abrir e fechar a conexão a cada query.
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(connection_options)
        .await
        .expect("Falha ao conectar no SQLite");

    // Garante que o diretório de uploads de imagens exista antes do servidor subir.
    std::fs::create_dir_all("uploads").expect("Falha ao criar diretório de uploads");
    
    // Inicializa as tabelas do banco de dados (migrations simples embutidas).
    init_db(&pool).await;

    // Estado da aplicação. O Axum permite compartilhar este 'state' de forma thread-safe 
    // entre todas as rotas (handlers). Aqui passamos nosso pool de banco de dados.
    let state = models::AppState { pool };
    
    // Configuração de CORS (Cross-Origin Resource Sharing).
    // Permite que os frontends (Site e Painel Admin) que rodam em portas/domínios diferentes
    // consumam a API. Em produção, você deve restringir 'Any' para os domínios específicos.
    let cors = CorsLayer::new().allow_origin(Any).allow_headers(Any).allow_methods(Any);

    // Configuração dos Servidores de Arquivos Estáticos (Frontend)
    // Usamos o 'ServeDir' para procurar os arquivos gerados pelo Vite (js, css, imagens).
    // O 'fallback' é essencial para SPAs: se o arquivo não existir (ex: o usuário acessou /sobre),
    // o servidor retorna o 'index.html' e deixa o Vue Router assumir a navegação no navegador.
    let admin_serve = ServeDir::new("../admin-canil/dist")
        .fallback(ServeFile::new("../admin-canil/dist/index.html"));

    let site_serve = ServeDir::new("../site/dist")
        .fallback(ServeFile::new("../site/dist/index.html"));

    // Construção das rotas (Router).
    // O Axum mapeia caminhos da URL para funções (handlers). 
    // 'nest_service' serve arquivos estáticos, enquanto 'route' atrela endpoints a verbos HTTP.
    let app = Router::new()
        .nest_service("/uploads", ServeDir::new("uploads"))
        .nest_service("/admin", admin_serve)
        .route("/api/auth/login", post(login_handler))
        .route("/api/admins", get(get_admins).post(create_admin))
        .route("/api/admins/:id", put(update_admin).delete(delete_admin))
        .route("/api/admins/:id/status", patch(update_admin_status))
        .route("/api/logs", get(get_system_logs))
        .route("/api/auth/password", patch(change_password))
        .route("/api/auth/preferences", patch(update_preferences))
        .route("/api/dashboard", get(get_dashboard))
        .route("/api/animais", get(get_animals).post(create_animal))
        .route("/api/animais/:id", get(get_animal).put(update_animal).patch(update_animal_status))
        .route("/api/animais/:id/tutores", post(toggle_tutorship))
        .route("/api/analytics/dashboard", get(get_dashboard_stats))
        .route("/api/analytics/sessions", get(get_sessions))
        .route("/api/analytics/sessions/:id", get(get_session_details))
        .route("/api/public/animais", get(get_public_animals))
        .route("/api/public/animais/:id", get(get_public_animal))
        .route("/api/public/analytics", post(register_event))
        .layer(cors)
        .with_state(state)
        .fallback_service(site_serve);

    // Define o endereço em que o servidor irá escutar (todas as interfaces na porta 8000).
    let addr = SocketAddr::from(([0, 0, 0, 0], 8000));
    println!("Servidor rodando em http://0.0.0.0:8000");
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    
    // Inicia o servidor recebendo conexões na porta vinculada.
    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>()).await.unwrap();
}

// INICIALIZAÇÃO DO BANCO
// Cria a estrutura SQL necessária se as tabelas não existirem. Em projetos maiores,
// é recomendado usar ferramentas de 'migrations' do próprio SQLx.
async fn init_db(pool: &sqlx::SqlitePool) {
    let schema = r#"
        CREATE TABLE IF NOT EXISTS admins (
            id INTEGER PRIMARY KEY AUTOINCREMENT, 
            name TEXT NOT NULL, 
            email TEXT NOT NULL UNIQUE, 
            phone TEXT NOT NULL, 
            password TEXT NOT NULL, 
            is_active INTEGER NOT NULL DEFAULT 1,
            is_master INTEGER NOT NULL DEFAULT 0,
            is_first_login INTEGER NOT NULL DEFAULT 1,
            pref_show_inactive INTEGER NOT NULL DEFAULT 0,
            pref_show_others INTEGER NOT NULL DEFAULT 0,
            pref_sort_by TEXT NOT NULL DEFAULT 'updated_desc',
            failed_attempts INTEGER NOT NULL DEFAULT 0,
            is_locked INTEGER NOT NULL DEFAULT 0
        );

        CREATE TABLE IF NOT EXISTS login_logs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            email TEXT NOT NULL,
            success INTEGER NOT NULL,
            remote_ip TEXT NOT NULL,
            severity TEXT NOT NULL DEFAULT "INFO",
            timestamp DATETIME DEFAULT CURRENT_TIMESTAMP
        );

        CREATE TABLE IF NOT EXISTS security_warnings (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            datetime DATETIME DEFAULT CURRENT_TIMESTAMP,
            msg TEXT NOT NULL,
            remote_ip TEXT NOT NULL,
            endpoint TEXT NOT NULL,
            user_agent TEXT NOT NULL,
            severity TEXT NOT NULL DEFAULT "CRITICAL"
        );

        CREATE TABLE IF NOT EXISTS action_logs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            admin_id INTEGER NOT NULL,
            action TEXT NOT NULL,
            severity TEXT NOT NULL DEFAULT "INFO",
            animal_id INTEGER,
            timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY(admin_id) REFERENCES admins(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS animals (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            species TEXT NOT NULL,
            birth_year INTEGER NOT NULL,
            breed TEXT,
            is_vaccinated INTEGER DEFAULT 0,
            is_dewormed INTEGER DEFAULT 0,
            behavior_dogs TEXT,
            behavior_cats TEXT,
            behavior_humans TEXT,
            independence TEXT,
            size TEXT,
            coat_color TEXT,
            coat_length TEXT,
            description TEXT,
            is_active INTEGER DEFAULT 1,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );

        CREATE TABLE IF NOT EXISTS animal_diseases (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            animal_id INTEGER NOT NULL,
            disease_name TEXT NOT NULL,
            FOREIGN KEY(animal_id) REFERENCES animals(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS animal_photos (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            animal_id INTEGER NOT NULL,
            file_path TEXT NOT NULL,
            is_primary INTEGER NOT NULL DEFAULT 0,
            is_active INTEGER NOT NULL DEFAULT 1,
            FOREIGN KEY(animal_id) REFERENCES animals(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS animal_tutors (
            animal_id INTEGER NOT NULL,
            admin_id INTEGER NOT NULL,
            PRIMARY KEY (animal_id, admin_id),
            FOREIGN KEY(animal_id) REFERENCES animals(id) ON DELETE CASCADE,
            FOREIGN KEY(admin_id) REFERENCES admins(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS site_analytics (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            visitor_id TEXT NOT NULL,
            ip_address TEXT,
            user_agent TEXT,
            event_type TEXT NOT NULL,
            path TEXT NOT NULL,
            animal_id INTEGER,
            payload TEXT,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );
    "#;

    // Roda a "planta do engenheiro" no banco de dados SQLite
    sqlx::query(schema).execute(pool).await.expect("Falha ao criar estrutura do banco");

    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM admins").fetch_one(pool).await.unwrap_or(0);
    if count == 0 {
    // Criação do administrador inicial (seed) caso o banco esteja completamente vazio.
    // Usamos bcrypt para hashear a senha antes de salvar no banco de dados. Isso é vital!
    // *Pitfall*: Nunca salve senhas em texto puro. Sempre use um algoritmo forte com salt (como bcrypt ou argon2).
        let default_password = bcrypt::hash("master", bcrypt::DEFAULT_COST).unwrap();
        sqlx::query("INSERT INTO admins (name, email, phone, password, is_first_login, is_master) VALUES (?, ?, ?, ?, 1, 1)")
            .bind("Administrador principal do sistema")
            .bind("master@master.master")
            .bind("")
            .bind(default_password)
            .execute(pool)
            .await
            .expect("Falha ao criar admin");
    }
}