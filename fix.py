import re

def rewrite(path):
    with open(path) as f:
        code = f.read()

    # ensure imports
    if 'ConnectInfo' not in code and 'axum::extract' in code:
        code = code.replace('extract::{', 'extract::{ConnectInfo, ')
    if 'std::net::SocketAddr' not in code:
        code = code.replace('use axum::{', 'use std::net::SocketAddr;\nuse axum::{')

    # Add ConnectInfo and HeaderMap before Json/Multipart/Query/Path if missing
    def repl_sig(m):
        sig = m.group(1)
        if 'ConnectInfo(addr)' not in sig:
            if 'mut multipart' in sig:
                return sig.replace('mut multipart', 'axum::extract::ConnectInfo(addr): axum::extract::ConnectInfo<SocketAddr>, headers: axum::http::HeaderMap, mut multipart')
            if 'Json(payload)' in sig:
                return sig.replace('Json(payload)', 'axum::extract::ConnectInfo(addr): axum::extract::ConnectInfo<SocketAddr>, headers: axum::http::HeaderMap, Json(payload)')
            if 'Path(id)' in sig and 'delete_animal_handler' in sig:
                return sig.replace('Path(id): Path<i64>,', 'Path(id): Path<i64>, axum::extract::ConnectInfo(addr): axum::extract::ConnectInfo<SocketAddr>, headers: axum::http::HeaderMap,')
            if 'toggle_tutorship' in sig:
                return sig.replace('Path(id): Path<i64>,', 'Path(id): Path<i64>, axum::extract::ConnectInfo(addr): axum::extract::ConnectInfo<SocketAddr>, headers: axum::http::HeaderMap,')
            if 'get_system_logs' in sig:
                return sig.replace('Query(filters)', 'axum::extract::ConnectInfo(addr): axum::extract::ConnectInfo<SocketAddr>, headers: axum::http::HeaderMap, Query(filters)')
            if 'delete_admin' in sig:
                return sig.replace('State(state): State<AppState>,', 'State(state): State<AppState>, axum::extract::ConnectInfo(addr): axum::extract::ConnectInfo<SocketAddr>, headers: axum::http::HeaderMap,')
        return sig

    code = re.sub(r'(pub async fn \w+\([\s\S]*?\)\s*->)', repl_sig, code)

    # Insert let ip
    def repl_body(m):
        body = m.group(1)
        if 'let ip =' not in body:
            return body.replace('{', '{\n    let ip = headers.get("x-forwarded-for").and_then(|v| v.to_str().ok()).map(|s| s.to_string()).unwrap_or_else(|| addr.ip().to_string());', 1)
        return body
        
    handlers = ["create_admin", "update_admin_status", "update_admin", "delete_admin", "change_password", "create_animal_handler", "update_animal_handler", "delete_animal_handler", "toggle_tutorship"]
    for h in handlers:
        code = re.sub(r'(pub async fn ' + h + r'[\s\S]*?\)\s*->\s*Result<[^>]+>\s*\{)', repl_body, code)

    # Update insert queries
    code = code.replace(
        'INSERT INTO action_logs (admin_id, action, severity) VALUES (?, ?, \\"WARNING\\")',
        'INSERT INTO action_logs (admin_id, action, severity, remote_ip) VALUES (?, ?, \\"WARNING\\", ?)'
    ).replace(
        'INSERT INTO action_logs (admin_id, action, severity) VALUES (?, ?, \\"CRITICAL\\")',
        'INSERT INTO action_logs (admin_id, action, severity, remote_ip) VALUES (?, ?, \\"CRITICAL\\", ?)'
    ).replace(
        "INSERT INTO action_logs (admin_id, action, severity, animal_id) VALUES (?, ?, 'WARNING', NULL)",
        "INSERT INTO action_logs (admin_id, action, severity, animal_id, remote_ip) VALUES (?, ?, 'WARNING', NULL, ?)"
    ).replace(
        "INSERT INTO action_logs (admin_id, action, severity, animal_id) VALUES (?, ?, 'WARNING', ?)",
        "INSERT INTO action_logs (admin_id, action, severity, animal_id, remote_ip) VALUES (?, ?, 'WARNING', ?, ?)"
    )

    # Update .bind()
    code = re.sub(r'(sqlx::query\("INSERT INTO action_logs[^;]*?\)\s*\n?\s*\.bind[^\)]*\)[^;]*?)\.execute', r'\1.bind(&ip).execute', code)
    
    # get_system_logs N/A removal
    code = code.replace(
        '\\"N/A\\" as remote_ip, timestamp FROM action_logs',
        'remote_ip, timestamp FROM action_logs'
    )
    
    with open(path, 'w') as f:
        f.write(code)

rewrite('api-canil/src/admin_handlers.rs')
rewrite('api-canil/src/handlers.rs')
