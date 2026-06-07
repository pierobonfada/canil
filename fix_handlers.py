import re

with open('api-canil/src/handlers.rs', 'r') as f:
    text = f.read()

# change_password
text = text.replace(
    '    Json(payload): Json<ChangePasswordRequest>,',
    '    axum::extract::ConnectInfo(addr): axum::extract::ConnectInfo<std::net::SocketAddr>,\n    headers: axum::http::HeaderMap,\n    Json(payload): Json<ChangePasswordRequest>,'
)
# create_animal_handler
text = text.replace(
    '    mut multipart: Multipart,',
    '    axum::extract::ConnectInfo(addr): axum::extract::ConnectInfo<std::net::SocketAddr>,\n    headers: axum::http::HeaderMap,\n    mut multipart: Multipart,'
)
# update_animal_handler
text = text.replace(
    '    Path(id): Path<i64>,\n    mut multipart: Multipart,',
    '    Path(id): Path<i64>,\n    axum::extract::ConnectInfo(addr): axum::extract::ConnectInfo<std::net::SocketAddr>,\n    headers: axum::http::HeaderMap,\n    mut multipart: Multipart,'
)
# delete_animal_handler
text = text.replace(
    '    Path(id): Path<i64>,\n) -> Result<Json<String>',
    '    Path(id): Path<i64>,\n    axum::extract::ConnectInfo(addr): axum::extract::ConnectInfo<std::net::SocketAddr>,\n    headers: axum::http::HeaderMap,\n) -> Result<Json<String>'
)
# toggle_tutorship
text = text.replace(
    '    Path(id): Path<i64>,\n) -> Result<Json<String>',
    '    Path(id): Path<i64>,\n    axum::extract::ConnectInfo(addr): axum::extract::ConnectInfo<std::net::SocketAddr>,\n    headers: axum::http::HeaderMap,\n) -> Result<Json<String>'
)

def add_ip_line(func_name, code):
    sig_match = re.search(r'pub async fn ' + func_name + r'[\s\S]*?\{', code)
    if sig_match:
        sig = sig_match.group(0)
        new_sig = sig + '\n    let ip = headers.get("x-forwarded-for").and_then(|v| v.to_str().ok()).unwrap_or_else(|| &addr.ip().to_string()).to_string();'
        return code.replace(sig, new_sig)
    return code

for fn in ['change_password', 'create_animal_handler', 'update_animal_handler', 'delete_animal_handler', 'toggle_tutorship']:
    text = add_ip_line(fn, text)

# Replace action_logs queries
text = text.replace(
    "INSERT INTO action_logs (admin_id, action, severity, animal_id) VALUES (?, ?, 'WARNING', NULL)",
    "INSERT INTO action_logs (admin_id, action, severity, remote_ip, animal_id) VALUES (?, ?, 'WARNING', ?, NULL)"
).replace(
    "INSERT INTO action_logs (admin_id, action, severity, animal_id) VALUES (?, ?, 'WARNING', ?)",
    "INSERT INTO action_logs (admin_id, action, severity, remote_ip, animal_id) VALUES (?, ?, 'WARNING', ?, ?)"
)

# And add .bind(&ip) BEFORE .execute or .fetch_one... Wait, in handlers.rs it binds claims.sub, action, and then executes...
# It's better to just do text replacement carefully.
# In change_password:
text = text.replace(
    '.bind("Redefiniu a senha").execute(&state.pool)',
    '.bind("Redefiniu a senha").bind(&ip).execute(&state.pool)'
)

# In create_animal_handler:
text = text.replace(
    '.bind(format!("Cadastrou o animal: {}", name)).bind(animal_id).execute(&state.pool)',
    '.bind(format!("Cadastrou o animal: {}", name)).bind(&ip).bind(animal_id).execute(&state.pool)'
)

# In update_animal_handler:
text = text.replace(
    '.bind(format!("Atualizou o perfil de {}", animal.name)).bind(id).execute(&state.pool)',
    '.bind(format!("Atualizou o perfil de {}", animal.name)).bind(&ip).bind(id).execute(&state.pool)'
)

# In delete_animal_handler:
text = text.replace(
    '.bind(format!("Desativou o animal de ID {}", id)).bind(id).execute(&state.pool)',
    '.bind(format!("Desativou o animal de ID {}", id)).bind(&ip).bind(id).execute(&state.pool)'
)

# In toggle_tutorship:
text = text.replace(
    '.bind(format!("Tornou-se tutor do animal {}", target.name)).bind(id).execute(&state.pool)',
    '.bind(format!("Tornou-se tutor do animal {}", target.name)).bind(&ip).bind(id).execute(&state.pool)'
).replace(
    '.bind(format!("Deixou de ser tutor do animal {}", target.name)).bind(id).execute(&state.pool)',
    '.bind(format!("Deixou de ser tutor do animal {}", target.name)).bind(&ip).bind(id).execute(&state.pool)'
)


with open('api-canil/src/handlers.rs', 'w') as f:
    f.write(text)

