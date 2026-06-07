import re

with open('api-canil/src/admin_handlers.rs', 'r') as f:
    text = f.read()

# Add extractors before Json
text = text.replace(
    '    Json(payload): Json<CreateAdminRequest>,',
    '    axum::extract::ConnectInfo(addr): axum::extract::ConnectInfo<std::net::SocketAddr>,\n    headers: axum::http::HeaderMap,\n    Json(payload): Json<CreateAdminRequest>,'
)
text = text.replace(
    '    Json(payload): Json<UpdateAdminStatusRequest>,',
    '    axum::extract::ConnectInfo(addr): axum::extract::ConnectInfo<std::net::SocketAddr>,\n    headers: axum::http::HeaderMap,\n    Json(payload): Json<UpdateAdminStatusRequest>,'
)
text = text.replace(
    '    Json(payload): Json<UpdateAdminRequest>,',
    '    axum::extract::ConnectInfo(addr): axum::extract::ConnectInfo<std::net::SocketAddr>,\n    headers: axum::http::HeaderMap,\n    Json(payload): Json<UpdateAdminRequest>,'
)
# For get_system_logs, Query is last
text = text.replace(
    '    Query(filters): Query<LogFilters>,',
    '    axum::extract::ConnectInfo(addr): axum::extract::ConnectInfo<std::net::SocketAddr>,\n    headers: axum::http::HeaderMap,\n    Query(filters): Query<LogFilters>,'
)
# For delete_admin, State is last
text = text.replace(
    '    State(state): State<AppState>,\n) -> Result<Json<String>',
    '    State(state): State<AppState>,\n    axum::extract::ConnectInfo(addr): axum::extract::ConnectInfo<std::net::SocketAddr>,\n    headers: axum::http::HeaderMap,\n) -> Result<Json<String>'
)

# Now inject let ip = ...
def inject_ip(func_name, code):
    return code.replace(
        f'pub async fn {func_name}(',
        f'pub async fn {func_name}('
    ).replace(
        f'{func_name}(' + re.search(f'{func_name}\\([\\s\\S]*?\\)\\s*->', code).group(0).split(') ->')[0].split('(')[1] + ') ->',
        f'{func_name}(' + re.search(f'{func_name}\\([\\s\\S]*?\\)\\s*->', code).group(0).split(') ->')[0].split('(')[1] + ') ->'
    )
    # Actually, simpler: find '{\n' after the function signature and add the ip line
    
def add_ip_line(func_name, code):
    sig_match = re.search(r'pub async fn ' + func_name + r'[\s\S]*?\{', code)
    if sig_match:
        sig = sig_match.group(0)
        new_sig = sig + '\n    let ip = headers.get("x-forwarded-for").and_then(|v| v.to_str().ok()).unwrap_or_else(|| &addr.ip().to_string()).to_string();'
        return code.replace(sig, new_sig)
    return code

for fn in ['create_admin', 'update_admin_status', 'update_admin', 'delete_admin', 'get_system_logs']:
    text = add_ip_line(fn, text)

# Now replace action_logs queries
text = text.replace(
    'INSERT INTO action_logs (admin_id, action, severity) VALUES (?, ?, \\"WARNING\\")',
    'INSERT INTO action_logs (admin_id, action, severity, remote_ip) VALUES (?, ?, \\"WARNING\\", ?)'
).replace(
    'INSERT INTO action_logs (admin_id, action, severity) VALUES (?, ?, \\"CRITICAL\\")',
    'INSERT INTO action_logs (admin_id, action, severity, remote_ip) VALUES (?, ?, \\"CRITICAL\\", ?)'
)
# Replace N/A
text = text.replace(
    '\\"N/A\\" as remote_ip, timestamp FROM action_logs',
    'remote_ip, timestamp FROM action_logs'
)

# And add .bind(&ip) before .execute
text = re.sub(r'(sqlx::query\("INSERT INTO action_logs[^;]+)\.execute', r'\1.bind(&ip).execute', text)

with open('api-canil/src/admin_handlers.rs', 'w') as f:
    f.write(text)

