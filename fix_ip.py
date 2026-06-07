import re

def process_file(filepath):
    with open(filepath, 'r') as f:
        content = f.read()

    # Add imports
    if 'ConnectInfo' not in content and 'axum::extract' in content:
        content = content.replace('extract::{', 'extract::{ConnectInfo, ')
    if 'std::net::SocketAddr' not in content:
        content = content.replace('use axum::{', 'use std::net::SocketAddr;\nuse axum::{')

    handlers = [
        "create_admin", "update_admin_status", "update_admin", "delete_admin",
        "change_password", "create_animal_handler", "update_animal_handler", 
        "delete_animal_handler", "toggle_tutorship"
    ]

    # Inject extractors correctly
    # Replace State(state): State<AppState>, with the extra extractors
    for handler in handlers:
        # We find the function definition and replace State(...) with State(...) + IP
        # We need a regex that matches the function name and the State up to the comma
        pattern = r'(pub async fn ' + handler + r'[\s\S]*?State\(state\):\s*State<AppState>,)'
        def repl_sig(m):
            sig = m.group(1)
            if 'ConnectInfo(addr)' not in sig:
                sig += '\n    ConnectInfo(addr): ConnectInfo<SocketAddr>,\n    headers: axum::http::HeaderMap,'
            return sig
        content = re.sub(pattern, repl_sig, content)
        
    # Replace INSERT statements
    content = content.replace(
        'INSERT INTO action_logs (admin_id, action, severity) VALUES (?, ?, ',
        'INSERT INTO action_logs (admin_id, action, severity, remote_ip) VALUES (?, ?, '
    ).replace(
        'INSERT INTO action_logs (admin_id, action, severity, animal_id) VALUES (?, ?, ',
        'INSERT INTO action_logs (admin_id, action, severity, remote_ip, animal_id) VALUES (?, ?, '
    ).replace(
        '"WARNING")',
        '"WARNING", ?)'
    ).replace(
        '"CRITICAL")',
        '"CRITICAL", ?)'
    ).replace(
        "'WARNING', NULL)",
        "'WARNING', ?, NULL)"
    ).replace(
        "'WARNING', ?)",
        "'WARNING', ?, ?)"
    )

    content = re.sub(r'(sqlx::query\("INSERT INTO action_logs[^;]+)\.execute', r'\1.bind(&ip).execute', content)
    content = re.sub(r'(sqlx::query\("INSERT INTO action_logs[^;]+)\.fetch_one', r'\1.bind(&ip).fetch_one', content)
    
    # Inject let ip = ...
    for handler in handlers:
        pattern_func = r'(pub async fn ' + handler + r'[\s\S]*?\)\s*->\s*Result<[^>]+>\s*\{)'
        def repl_func(m):
            header = m.group(1)
            ip_line = '\n    let ip = headers.get("x-forwarded-for").and_then(|v| v.to_str().ok()).unwrap_or_else(|| &addr.ip().to_string()).to_string();'
            if 'let ip =' not in header + ip_line:
                pass
            return header + ip_line
        
        if handler in content:
            content = re.sub(pattern_func, repl_func, content)

    # In admin_handlers.rs, we also need to change get_system_logs query
    if 'get_system_logs' in content:
        content = content.replace(
            r'\"N/A\" as remote_ip, timestamp FROM action_logs',
            r'remote_ip, timestamp FROM action_logs'
        )

    with open(filepath, 'w') as f:
        f.write(content)

process_file('api-canil/src/admin_handlers.rs')
process_file('api-canil/src/handlers.rs')

