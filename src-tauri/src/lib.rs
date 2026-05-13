use serde::{Deserialize, Serialize};
#[cfg(target_os = "windows")]
use std::ffi::OsStr;
#[cfg(target_os = "windows")]
use std::os::windows::ffi::OsStrExt;
#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;
use std::{
    collections::BTreeMap,
    fs,
    io::{BufRead, BufReader, Read},
    net::{TcpListener, TcpStream},
    path::PathBuf,
    process::{Command, Stdio},
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager, State, WindowEvent, Wry,
};
#[cfg(target_os = "windows")]
use windows_sys::Win32::{
    Foundation::CloseHandle,
    System::Threading::{WaitForSingleObject, INFINITE},
    UI::{
        Shell::{ShellExecuteExW, SEE_MASK_NOCLOSEPROCESS, SHELLEXECUTEINFOW},
        WindowsAndMessaging::SW_HIDE,
    },
};

#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;

#[derive(Clone)]
struct AppState {
    store: Arc<Mutex<ClientStore>>,
    tray_toggle: Arc<Mutex<Option<MenuItem<Wry>>>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            store: Arc::new(Mutex::new(ClientStore::new())),
            tray_toggle: Arc::new(Mutex::new(None)),
        }
    }
}

struct ClientStore {
    profile: ClientProfile,
    servers: Vec<ServerProfile>,
    status: ClientStatus,
    runtime_pid: Option<u32>,
    tun_active: bool,
    system_proxy_backup: Option<SystemProxyBackup>,
}

impl ClientStore {
    fn new() -> Self {
        let config = load_config().unwrap_or_default();
        let profile = config.profile;
        Self {
            status: ClientStatus::ready(&profile),
            profile,
            servers: config.servers,
            runtime_pid: None,
            tun_active: false,
            system_proxy_backup: None,
        }
    }

    fn append_log(&mut self, line: impl AsRef<str>) {
        self.status
            .logs
            .push(format!("{}  {}", time_stamp(), line.as_ref()));
        if self.status.logs.len() > 120 {
            let keep_from = self.status.logs.len() - 120;
            self.status.logs = self.status.logs.split_off(keep_from);
        }
    }
}

#[derive(Clone, Debug)]
struct SystemProxyBackup {
    proxy_enable: Option<String>,
    proxy_server: Option<String>,
    proxy_override: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct ClientProfile {
    name: String,
    subscription_url: String,
    uri: String,
    mode: String,
    #[serde(default = "default_language")]
    language: String,
    #[serde(default = "default_proxy_auto")]
    proxy_auto: bool,
    socks_host: String,
    socks_port: u16,
    dns: String,
    olcrtc_path: String,
    #[serde(default)]
    welcome_dismissed: bool,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
struct ClientConfig {
    profile: ClientProfile,
    servers: Vec<ServerProfile>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct ServerProfile {
    id: String,
    name: String,
    uri: String,
    carrier: String,
    transport: String,
}

impl Default for ClientProfile {
    fn default() -> Self {
        Self {
            name: "Default".into(),
            subscription_url: String::new(),
            uri: String::new(),
            mode: "proxy".into(),
            language: default_language(),
            proxy_auto: default_proxy_auto(),
            socks_host: "127.0.0.1".into(),
            socks_port: 0,
            dns: "1.1.1.1:53".into(),
            olcrtc_path: "olcrtc.exe".into(),
            welcome_dismissed: false,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
struct ClientStatus {
    state: String,
    mode: String,
    socks: String,
    download_bps: u64,
    upload_bps: u64,
    download_bytes: u64,
    upload_bytes: u64,
    started_at: String,
    notice: String,
    target: Option<PublicTarget>,
    steps: Vec<String>,
    logs: Vec<String>,
}

impl ClientStatus {
    fn ready(profile: &ClientProfile) -> Self {
        Self {
            state: "disconnected".into(),
            mode: profile.mode.clone(),
            socks: socks_address(profile),
            download_bps: 0,
            upload_bps: 0,
            download_bytes: 0,
            upload_bytes: 0,
            started_at: String::new(),
            notice: String::new(),
            target: None,
            steps: planned_steps(&profile.mode),
            logs: vec!["Client ready".into()],
        }
    }
}

#[derive(Clone, Debug, Serialize)]
struct PublicTarget {
    carrier: String,
    transport: String,
    name: String,
}

#[derive(Clone, Debug)]
struct RuntimeTarget {
    carrier: String,
    transport: String,
    room_id: String,
    key: String,
    client_id: String,
    name: String,
    payload: BTreeMap<String, String>,
}

impl RuntimeTarget {
    fn public(&self) -> PublicTarget {
        PublicTarget {
            carrier: self.carrier.clone(),
            transport: self.transport.clone(),
            name: self.name.clone(),
        }
    }
}

#[tauri::command]
fn get_profile(state: State<AppState>) -> Result<ClientProfile, String> {
    Ok(state.store.lock().map_err(lock_error)?.profile.clone())
}

#[tauri::command]
fn save_profile(profile: ClientProfile, state: State<AppState>) -> Result<ClientProfile, String> {
    let profile = normalize_profile(profile);
    validate_saved_profile(&profile)?;

    {
        let store = state.store.lock().map_err(lock_error)?;
        save_config_file(&ClientConfig {
            profile: profile.clone(),
            servers: store.servers.clone(),
        })?;
    }

    let mut store = state.store.lock().map_err(lock_error)?;
    store.profile = profile.clone();
    store.status.mode = profile.mode.clone();
    store.status.socks = socks_address(&profile);
    store.status.steps = planned_steps(&profile.mode);
    store.status.notice.clear();
    store.append_log(format!("Profile saved: {}", profile.name));
    Ok(profile)
}

#[tauri::command]
fn dismiss_welcome(state: State<AppState>) -> Result<ClientProfile, String> {
    let mut store = state.store.lock().map_err(lock_error)?;
    store.profile.welcome_dismissed = true;
    save_config_file(&ClientConfig {
        profile: store.profile.clone(),
        servers: store.servers.clone(),
    })?;
    store.append_log("Welcome screen dismissed");
    Ok(store.profile.clone())
}

#[tauri::command]
fn check_public_internet() -> bool {
    let client = match reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
    {
        Ok(client) => client,
        Err(_) => return false,
    };

    for url in [
        "https://www.google.com/generate_204",
        "https://api.ipify.org",
    ] {
        if client
            .get(url)
            .send()
            .and_then(|response| response.error_for_status())
            .is_ok()
        {
            return true;
        }
    }
    false
}

#[tauri::command]
fn get_servers(state: State<AppState>) -> Result<Vec<ServerProfile>, String> {
    Ok(state.store.lock().map_err(lock_error)?.servers.clone())
}

#[tauri::command]
fn import_servers(raw: String, state: State<AppState>) -> Result<Vec<ServerProfile>, String> {
    let mut imported = Vec::new();
    for line in raw.lines() {
        let value = line.trim();
        if value.is_empty() || value.starts_with('#') {
            continue;
        }
        let target = parse_runtime_uri(value)?;
        imported.push(ServerProfile {
            id: server_id(value),
            name: server_name(&target),
            uri: value.to_string(),
            carrier: target.carrier,
            transport: target.transport,
        });
    }
    if imported.is_empty() {
        return Err("No olcrtc links found".into());
    }

    let mut store = state.store.lock().map_err(lock_error)?;
    for server in imported {
        if let Some(existing) = store.servers.iter_mut().find(|item| item.id == server.id) {
            *existing = server;
        } else {
            store.servers.push(server);
        }
    }
    store
        .servers
        .sort_by(|left, right| left.name.cmp(&right.name));
    if store.profile.uri.is_empty() {
        if let Some(first) = store.servers.first().cloned() {
            store.profile.uri = first.uri;
            store.profile.name = first.name;
        }
    }
    save_config_file(&ClientConfig {
        profile: store.profile.clone(),
        servers: store.servers.clone(),
    })?;
    store.append_log("Profiles imported");
    Ok(store.servers.clone())
}

#[tauri::command]
fn select_server(id: String, state: State<AppState>) -> Result<ClientProfile, String> {
    let mut store = state.store.lock().map_err(lock_error)?;
    let server = store
        .servers
        .iter()
        .find(|server| server.id == id)
        .cloned()
        .ok_or_else(|| "Server profile not found".to_string())?;
    store.profile.name = server.name;
    store.profile.uri = server.uri;
    store.profile.subscription_url.clear();
    save_config_file(&ClientConfig {
        profile: store.profile.clone(),
        servers: store.servers.clone(),
    })?;
    store.status.mode = store.profile.mode.clone();
    store.status.socks = socks_address(&store.profile);
    store.status.steps = planned_steps(&store.profile.mode);
    let profile_name = store.profile.name.clone();
    store.append_log(format!("Selected profile: {profile_name}"));
    Ok(store.profile.clone())
}

#[tauri::command]
fn delete_server(id: String, state: State<AppState>) -> Result<Vec<ServerProfile>, String> {
    let mut store = state.store.lock().map_err(lock_error)?;
    let before = store.servers.len();
    store.servers.retain(|server| server.id != id);
    if store.servers.len() == before {
        return Err("Server profile not found".into());
    }

    let active_exists = store
        .servers
        .iter()
        .any(|server| server.uri == store.profile.uri);
    if !active_exists {
        if let Some(first) = store.servers.first().cloned() {
            store.profile.name = first.name;
            store.profile.uri = first.uri;
        } else {
            store.profile.name = "Default".into();
            store.profile.uri.clear();
            store.profile.subscription_url.clear();
        }
    }

    save_config_file(&ClientConfig {
        profile: store.profile.clone(),
        servers: store.servers.clone(),
    })?;
    store.append_log("Profile deleted");
    Ok(store.servers.clone())
}

#[tauri::command]
fn get_status(state: State<AppState>) -> Result<ClientStatus, String> {
    Ok(state.store.lock().map_err(lock_error)?.status.clone())
}

#[tauri::command]
fn validate_uri(raw: String) -> Result<(), String> {
    parse_runtime_uri(&raw).map(|_| ())
}

#[tauri::command]
fn window_minimize(window: tauri::Window) -> Result<(), String> {
    window.minimize().map_err(|err| err.to_string())
}

#[tauri::command]
fn window_hide_to_tray(window: tauri::Window) -> Result<(), String> {
    window.hide().map_err(|err| err.to_string())
}

#[tauri::command]
fn window_start_dragging(window: tauri::Window) -> Result<(), String> {
    window.start_dragging().map_err(|err| err.to_string())
}

#[tauri::command]
fn open_external(url: String) -> Result<(), String> {
    const ALLOWED: &[&str] = &[
        "https://t.me/svllvsxprod",
        "https://t.me/openlibrecommunity",
        "https://t.me/tribute/app?startapp=dK9j",
        "https://nowpayments.io/donation/svllvsx",
    ];
    if !ALLOWED.contains(&url.as_str()) {
        return Err("external URL is not allowed".into());
    }

    #[cfg(target_os = "windows")]
    {
        let mut command = Command::new("rundll32.exe");
        hide_window(&mut command);
        command
            .args(["url.dll,FileProtocolHandler", &url])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|err| format!("open browser: {err}"))?;
    }
    #[cfg(not(target_os = "windows"))]
    {
        let mut command = Command::new("xdg-open");
        command
            .arg(&url)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|err| format!("open browser: {err}"))?;
    }
    Ok(())
}

#[tauri::command]
fn connect(state: State<AppState>) -> Result<ClientStatus, String> {
    connect_inner(state.inner())
}

fn connect_inner(state: &AppState) -> Result<ClientStatus, String> {
    let profile = {
        let store = state.store.lock().map_err(lock_error)?;
        if store.runtime_pid.is_some() {
            return Ok(store.status.clone());
        }
        normalize_profile(store.profile.clone())
    };

    validate_profile(&profile)?;
    {
        let mut store = state.store.lock().map_err(lock_error)?;
        store.status.state = "connecting".into();
        store.status.notice.clear();
        store.append_log(format!("Resolving profile {}", profile.name));
    }

    let target = resolve_runtime_target(&profile)?;
    let runtime_profile = resolve_proxy(&profile)?;
    let data_dir = ensure_data_dir()?;
    let args = olcrtc_args(&runtime_profile, &target, &data_dir);
    let tun_requested = runtime_profile.mode == "tun";

    let mut command = Command::new(&runtime_profile.olcrtc_path);
    hide_window(&mut command);
    let mut child = command
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|err| format!("start olcrtc: {err}"))?;

    let pid = child.id();
    let stdout = child.stdout.take();
    let stderr = child.stderr.take();
    let store_for_wait = state.store.clone();

    if let Err(err) = wait_for_tcp(&runtime_profile.socks_host, runtime_profile.socks_port) {
        kill_process(pid);
        return Err(err);
    }

    let tun_active = if tun_requested {
        match start_net_service_tun(&runtime_profile) {
            Ok(()) => true,
            Err(err) => {
                kill_process(pid);
                return Err(err);
            }
        }
    } else {
        false
    };

    {
        let mut store = state.store.lock().map_err(lock_error)?;
        store.profile = profile.clone();
        if tun_requested {
            store.append_log("System proxy left unchanged; TUN routes system traffic directly");
        }
        store.runtime_pid = Some(pid);
        store.tun_active = tun_active;
        store.status.state = "connected".into();
        store.status.mode = runtime_profile.mode.clone();
        store.status.socks = socks_address(&runtime_profile);
        store.status.started_at = unix_seconds().to_string();
        store.status.target = Some(target.public());
        store.status.steps = planned_steps(&runtime_profile.mode);
        store.append_log(format!(
            "Started {}/{} at {}",
            target.carrier,
            target.transport,
            socks_address(&runtime_profile)
        ));
        if tun_requested {
            store.append_log(
                "TUN mode selected: LibreRTC Net Service started embedded sing-box TUN",
            );
        } else {
            store.append_log("Proxy mode selected");
        }
    }

    if let Some(stdout) = stdout {
        stream_logs(stdout, state.store.clone());
    }
    if let Some(stderr) = stderr {
        stream_logs(stderr, state.store.clone());
    }

    thread::spawn(move || {
        let result = child.wait();
        if let Ok(mut store) = store_for_wait.lock() {
            if store.runtime_pid == Some(pid) {
                store.runtime_pid = None;
                if store.tun_active {
                    store.tun_active = false;
                    if let Err(err) = stop_net_service_tun() {
                        store.append_log(format!("TUN service stop failed: {err}"));
                    }
                }
                if let Some(backup) = store.system_proxy_backup.take() {
                    if let Err(err) = restore_system_proxy(&backup) {
                        store.append_log(format!("System proxy restore failed: {err}"));
                    } else {
                        store.append_log("System proxy restored");
                    }
                }
                store.status.state = "disconnected".into();
                store.status.started_at.clear();
                store.status.download_bps = 0;
                store.status.upload_bps = 0;
                match result {
                    Ok(status) => store.append_log(format!("olcrtc exited with {status}")),
                    Err(err) => store.append_log(format!("olcrtc wait failed: {err}")),
                }
            }
        }
    });

    let status = current_status(state)?;
    update_tray_toggle(state, &status);
    Ok(status)
}

#[tauri::command]
fn disconnect(state: State<AppState>) -> Result<ClientStatus, String> {
    disconnect_inner(state.inner())
}

fn disconnect_inner(state: &AppState) -> Result<ClientStatus, String> {
    let (pid, tun_active, proxy_backup) = {
        let mut store = state.store.lock().map_err(lock_error)?;
        store.append_log("Disconnect requested");
        let pid = store.runtime_pid.take();
        let tun_active = store.tun_active;
        store.tun_active = false;
        let proxy_backup = store.system_proxy_backup.take();
        store.status.state = "disconnected".into();
        store.status.started_at.clear();
        store.status.notice.clear();
        store.status.download_bps = 0;
        store.status.upload_bps = 0;
        (pid, tun_active, proxy_backup)
    };

    if let Some(backup) = proxy_backup {
        restore_system_proxy(&backup)?;
    }
    if tun_active {
        stop_net_service_tun()?;
    }
    if let Some(pid) = pid {
        kill_process(pid);
    }

    let status = current_status(state)?;
    update_tray_toggle(state, &status);
    Ok(status)
}

fn current_status(state: &AppState) -> Result<ClientStatus, String> {
    Ok(state.store.lock().map_err(lock_error)?.status.clone())
}

fn normalize_profile(mut profile: ClientProfile) -> ClientProfile {
    profile.name = profile.name.trim().to_string();
    profile.subscription_url = profile.subscription_url.trim().to_string();
    profile.uri = profile.uri.trim().to_string();
    profile.mode = profile.mode.trim().to_lowercase();
    profile.language = profile.language.trim().to_lowercase();
    profile.socks_host = profile.socks_host.trim().to_string();
    profile.dns = profile.dns.trim().to_string();
    profile.olcrtc_path = profile.olcrtc_path.trim().to_string();

    if profile.name.is_empty() {
        profile.name = "Default".into();
    }
    if profile.mode != "tun" {
        profile.mode = "proxy".into();
    }
    if profile.language != "ru" {
        profile.language = default_language();
    }
    if profile.socks_host.is_empty() {
        profile.socks_host = "127.0.0.1".into();
    }
    if !profile.proxy_auto && profile.socks_port == 0 {
        profile.socks_port = 8808;
    }
    if profile.dns.is_empty() {
        profile.dns = "1.1.1.1:53".into();
    }
    if profile.olcrtc_path.is_empty() {
        profile.olcrtc_path = "olcrtc.exe".into();
    }
    profile
}

fn validate_profile(profile: &ClientProfile) -> Result<(), String> {
    if !profile.proxy_auto && profile.socks_port == 0 {
        return Err("SOCKS port must be between 1 and 65535".into());
    }
    if profile.subscription_url.is_empty() && profile.uri.is_empty() {
        return Err("Subscription URL or direct URI is required".into());
    }
    if !profile.uri.is_empty() {
        parse_runtime_uri(&profile.uri)?;
    }
    Ok(())
}

fn validate_saved_profile(profile: &ClientProfile) -> Result<(), String> {
    if !profile.proxy_auto && profile.socks_port == 0 {
        return Err("SOCKS port must be between 1 and 65535".into());
    }
    if !profile.uri.is_empty() {
        parse_runtime_uri(&profile.uri)?;
    }
    Ok(())
}

fn resolve_runtime_target(profile: &ClientProfile) -> Result<RuntimeTarget, String> {
    if !profile.uri.trim().is_empty() {
        return parse_runtime_uri(&profile.uri);
    }
    if profile.subscription_url.trim().is_empty() {
        return Err("Subscription URL or direct URI is required".into());
    }

    let body = reqwest::blocking::get(&profile.subscription_url)
        .and_then(|response| response.error_for_status())
        .map_err(|err| format!("fetch subscription: {err}"))?
        .text()
        .map_err(|err| format!("read subscription: {err}"))?;

    for line in body.lines() {
        let value = line.trim();
        if value.is_empty() || value.starts_with('#') {
            continue;
        }
        return parse_runtime_uri(value);
    }
    Err("Subscription does not contain an olcrtc URI".into())
}

fn resolve_proxy(profile: &ClientProfile) -> Result<ClientProfile, String> {
    let mut profile = profile.clone();
    if profile.proxy_auto {
        let listener = TcpListener::bind((profile.socks_host.as_str(), 0))
            .map_err(|err| format!("choose local proxy port: {err}"))?;
        profile.socks_port = listener
            .local_addr()
            .map_err(|err| format!("read local proxy port: {err}"))?
            .port();
        drop(listener);
    }
    Ok(profile)
}

fn parse_runtime_uri(raw: &str) -> Result<RuntimeTarget, String> {
    let raw = raw.trim();
    if raw.is_empty() {
        return Err("URI is required".into());
    }
    let rest = raw
        .strip_prefix("olcrtc://")
        .ok_or_else(|| "URI must start with olcrtc://".to_string())?;
    let (carrier, rest) = rest
        .split_once('?')
        .ok_or_else(|| "URI carrier is required".to_string())?;
    if carrier.is_empty() {
        return Err("URI carrier is required".into());
    }
    let (before_secret, secret) = rest
        .split_once('#')
        .ok_or_else(|| "URI key is required".to_string())?;
    let (left, room_id) = before_secret
        .split_once('@')
        .ok_or_else(|| "URI room ID is required".to_string())?;
    if room_id.is_empty() {
        return Err("URI room ID is required".into());
    }
    let (key, tail) = secret
        .split_once('%')
        .ok_or_else(|| "URI shared key is required".to_string())?;
    if key.is_empty() {
        return Err("URI shared key is required".into());
    }
    let (client_id, name) = tail.split_once('$').unwrap_or((tail, ""));
    if client_id.is_empty() {
        return Err("URI client ID is required".into());
    }

    let mut transport = left.to_string();
    let mut payload = BTreeMap::new();
    if let Some(start) = left.find('<') {
        let end = left
            .rfind('>')
            .ok_or_else(|| "URI payload is invalid".to_string())?;
        if end <= start {
            return Err("URI payload is invalid".into());
        }
        transport = left[..start].to_string();
        for item in left[start + 1..end].split('&') {
            if item.is_empty() {
                continue;
            }
            let (key, value) = item
                .split_once('=')
                .ok_or_else(|| "URI payload item is invalid".to_string())?;
            if key.is_empty() {
                return Err("URI payload item is invalid".into());
            }
            payload.insert(key.to_string(), value.to_string());
        }
    }
    if transport.is_empty() {
        return Err("URI transport is required".into());
    }

    Ok(RuntimeTarget {
        carrier: carrier.to_string(),
        transport,
        room_id: room_id.to_string(),
        key: key.to_string(),
        client_id: client_id.to_string(),
        name: name.to_string(),
        payload,
    })
}

fn ensure_data_dir() -> Result<PathBuf, String> {
    let base = std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    let data_dir = base.join("LibreRTC").join("data");
    fs::create_dir_all(&data_dir).map_err(|err| format!("create data directory: {err}"))?;
    Ok(data_dir)
}

fn profile_path() -> Result<PathBuf, String> {
    let base = std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    let dir = base.join("LibreRTC").join("client");
    fs::create_dir_all(&dir).map_err(|err| format!("create profile directory: {err}"))?;
    Ok(dir.join("profile.json"))
}

fn load_config() -> Option<ClientConfig> {
    let path = profile_path().ok()?;
    let raw = fs::read_to_string(path).ok()?;
    if let Ok(mut config) = serde_json::from_str::<ClientConfig>(&raw) {
        config.profile = normalize_profile(config.profile);
        return Some(config);
    }
    let profile = serde_json::from_str::<ClientProfile>(&raw).ok()?;
    Some(ClientConfig {
        profile: normalize_profile(profile),
        servers: Vec::new(),
    })
}

fn save_config_file(config: &ClientConfig) -> Result<(), String> {
    let path = profile_path()?;
    let raw =
        serde_json::to_string_pretty(config).map_err(|err| format!("encode profile: {err}"))?;
    fs::write(path, raw).map_err(|err| format!("save profile: {err}"))
}

fn server_id(uri: &str) -> String {
    format!("{:016x}", fnv1a64(uri.as_bytes()))
}

fn server_name(target: &RuntimeTarget) -> String {
    if target.name.trim().is_empty() {
        format!("{}/{}", target.carrier, target.transport)
    } else {
        target.name.clone()
    }
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

fn olcrtc_args(profile: &ClientProfile, target: &RuntimeTarget, data_dir: &PathBuf) -> Vec<String> {
    let mut args = vec![
        "-mode".into(),
        "cnc".into(),
        "-carrier".into(),
        target.carrier.clone(),
        "-transport".into(),
        target.transport.clone(),
        "-id".into(),
        target.room_id.clone(),
        "-client-id".into(),
        target.client_id.clone(),
        "-key".into(),
        target.key.clone(),
        "-link".into(),
        "direct".into(),
        "-data".into(),
        data_dir.to_string_lossy().into_owned(),
        "-dns".into(),
        profile.dns.clone(),
        "-socks-host".into(),
        profile.socks_host.clone(),
        "-socks-port".into(),
        profile.socks_port.to_string(),
        "-stats-interval".into(),
        "1000".into(),
    ];
    for (key, value) in &target.payload {
        args.push(format!("-{key}"));
        args.push(value.clone());
    }
    args
}

fn wait_for_tcp(host: &str, port: u16) -> Result<(), String> {
    let deadline = Instant::now() + Duration::from_secs(12);
    let address = format!("{host}:{port}");
    while Instant::now() < deadline {
        if TcpStream::connect(&address).is_ok() {
            return Ok(());
        }
        thread::sleep(Duration::from_millis(150));
    }
    Err(format!(
        "SOCKS runtime did not start listening at {address}"
    ))
}

fn hide_window(command: &mut Command) {
    #[cfg(target_os = "windows")]
    {
        command.creation_flags(CREATE_NO_WINDOW);
    }
}

#[derive(Serialize)]
struct NetServiceStartRequest {
    socks_host: String,
    socks_port: u16,
    dns: String,
}

fn start_net_service_tun(profile: &ClientProfile) -> Result<(), String> {
    ensure_net_service_running()?;
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(20))
        .build()
        .map_err(|err| format!("create net service client: {err}"))?;
    let response = client
        .post("http://127.0.0.1:38741/start")
        .json(&NetServiceStartRequest {
            socks_host: profile.socks_host.clone(),
            socks_port: profile.socks_port,
            dns: profile.dns.clone(),
        })
        .send()
        .map_err(|err| {
            format!(
                "LibreRTC Net Service is not available. Install/start librertc-net-service.exe: {err}"
            )
        })?;
    if response.status().is_success() {
        Ok(())
    } else {
        let status = response.status();
        let body = response.text().unwrap_or_default();
        Err(format!(
            "LibreRTC Net Service start failed ({status}): {body}"
        ))
    }
}

fn ensure_net_service_running() -> Result<(), String> {
    if net_service_health().is_ok() {
        return Ok(());
    }
    install_net_service()?;
    let deadline = Instant::now() + Duration::from_secs(20);
    while Instant::now() < deadline {
        if net_service_health().is_ok() {
            return Ok(());
        }
        thread::sleep(Duration::from_millis(300));
    }
    Err("LibreRTC Net Service did not become ready after installation".into())
}

fn net_service_health() -> Result<(), String> {
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(2))
        .build()
        .map_err(|err| format!("create net service client: {err}"))?;
    let response = client
        .get("http://127.0.0.1:38741/health")
        .send()
        .map_err(|err| format!("check LibreRTC Net Service health: {err}"))?;
    if response.status().is_success() {
        Ok(())
    } else {
        Err(format!(
            "LibreRTC Net Service health failed: {}",
            response.status()
        ))
    }
}

fn install_net_service() -> Result<(), String> {
    let service_path = resolve_net_service_path()?;
    #[cfg(target_os = "windows")]
    {
        shell_execute_elevated_wait(&service_path, "-install")
            .map_err(|err| format!("install LibreRTC Net Service: {err}"))
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = service_path;
        Err("LibreRTC Net Service bootstrap is only implemented on Windows".into())
    }
}

fn resolve_net_service_path() -> Result<PathBuf, String> {
    let exe = std::env::current_exe().map_err(|err| format!("resolve app path: {err}"))?;
    let dir = exe
        .parent()
        .ok_or_else(|| "resolve app directory: executable has no parent".to_string())?;
    let service = dir.join("librertc-net-service.exe");
    if service.is_file() {
        Ok(service)
    } else {
        Err(format!(
            "TUN mode requires {} next to librertc-client.exe",
            service.display()
        ))
    }
}

#[cfg(target_os = "windows")]
fn shell_execute_elevated_wait(file: &std::path::Path, parameters: &str) -> Result<(), String> {
    let verb = wide_null(OsStr::new("runas"));
    let file = wide_null(file.as_os_str());
    let parameters = wide_null(OsStr::new(parameters));
    let mut info: SHELLEXECUTEINFOW = unsafe { std::mem::zeroed() };
    info.cbSize = std::mem::size_of::<SHELLEXECUTEINFOW>() as u32;
    info.fMask = SEE_MASK_NOCLOSEPROCESS;
    info.lpVerb = verb.as_ptr();
    info.lpFile = file.as_ptr();
    info.lpParameters = parameters.as_ptr();
    info.nShow = SW_HIDE;
    let ok = unsafe { ShellExecuteExW(&mut info) };
    if ok == 0 {
        return Err("administrator prompt was cancelled or denied".into());
    }
    if info.hProcess.is_null() {
        return Err("administrator process started without a process handle".into());
    }
    unsafe {
        WaitForSingleObject(info.hProcess, INFINITE);
        CloseHandle(info.hProcess);
    }
    Ok(())
}

#[cfg(target_os = "windows")]
fn wide_null(value: impl AsRef<OsStr>) -> Vec<u16> {
    value.as_ref().encode_wide().chain(Some(0)).collect()
}

fn stop_net_service_tun() -> Result<(), String> {
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .map_err(|err| format!("create net service client: {err}"))?;
    let response = client
        .post("http://127.0.0.1:38741/stop")
        .send()
        .map_err(|err| format!("stop LibreRTC Net Service TUN: {err}"))?;
    if response.status().is_success() {
        Ok(())
    } else {
        let status = response.status();
        let body = response.text().unwrap_or_default();
        Err(format!(
            "LibreRTC Net Service stop failed ({status}): {body}"
        ))
    }
}

fn stream_logs<R>(reader: R, store: Arc<Mutex<ClientStore>>)
where
    R: Read + Send + 'static,
{
    thread::spawn(move || {
        for line in BufReader::new(reader).lines() {
            match line {
                Ok(line) => {
                    if let Ok(mut store) = store.lock() {
                        if handle_stats_line(&line, &mut store) {
                            continue;
                        }
                        store.append_log(line);
                    }
                }
                Err(err) => {
                    if let Ok(mut store) = store.lock() {
                        store.append_log(format!("Log stream error: {err}"));
                    }
                    break;
                }
            }
        }
    });
}

#[derive(Deserialize)]
struct RuntimeStatsLine {
    download_bps: u64,
    upload_bps: u64,
    download_bytes: u64,
    upload_bytes: u64,
}

fn handle_stats_line(line: &str, store: &mut ClientStore) -> bool {
    let Some(raw) = line.strip_prefix("OLCRTC_STATS ") else {
        return false;
    };
    let Ok(stats) = serde_json::from_str::<RuntimeStatsLine>(raw) else {
        return false;
    };
    store.status.download_bps = stats.download_bps;
    store.status.upload_bps = stats.upload_bps;
    store.status.download_bytes = stats.download_bytes;
    store.status.upload_bytes = stats.upload_bytes;
    true
}

fn kill_process(pid: u32) {
    #[cfg(target_os = "windows")]
    {
        let mut command = Command::new("taskkill");
        hide_window(&mut command);
        let _ = command
            .args(["/PID", &pid.to_string(), "/T", "/F"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = Command::new("kill")
            .args(["-TERM", &pid.to_string()])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    }
}

fn startup_cleanup() {
    let _ = stop_net_service_tun();
    kill_process_by_name("olcrtc.exe");
}

fn kill_process_by_name(name: &str) {
    #[cfg(target_os = "windows")]
    {
        let mut command = Command::new("taskkill");
        hide_window(&mut command);
        let _ = command
            .args(["/IM", name, "/T", "/F"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = name;
    }
}

fn restore_system_proxy(backup: &SystemProxyBackup) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        restore_reg_value("ProxyEnable", "REG_DWORD", backup.proxy_enable.as_deref())?;
        restore_reg_value("ProxyServer", "REG_SZ", backup.proxy_server.as_deref())?;
        restore_reg_value("ProxyOverride", "REG_SZ", backup.proxy_override.as_deref())?;
        notify_proxy_change();
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = backup;
    }

    Ok(())
}

#[cfg(target_os = "windows")]
fn reg_set(name: &str, kind: &str, value: &str) -> Result<(), String> {
    let mut command = Command::new("reg");
    hide_window(&mut command);
    let status = command
        .args([
            "add",
            r"HKCU\Software\Microsoft\Windows\CurrentVersion\Internet Settings",
            "/v",
            name,
            "/t",
            kind,
            "/d",
            value,
            "/f",
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .status()
        .map_err(|err| format!("set {name}: {err}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("set {name}: reg exited with {status}"))
    }
}

#[cfg(target_os = "windows")]
fn reg_delete(name: &str) -> Result<(), String> {
    let mut command = Command::new("reg");
    hide_window(&mut command);
    let status = command
        .args([
            "delete",
            r"HKCU\Software\Microsoft\Windows\CurrentVersion\Internet Settings",
            "/v",
            name,
            "/f",
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_err(|err| format!("delete {name}: {err}"))?;
    if status.success() {
        Ok(())
    } else {
        Ok(())
    }
}

#[cfg(target_os = "windows")]
fn restore_reg_value(name: &str, kind: &str, value: Option<&str>) -> Result<(), String> {
    match value {
        Some(value) => reg_set(name, kind, value),
        None => reg_delete(name),
    }
}

#[cfg(target_os = "windows")]
fn notify_proxy_change() {
    let script = r#"
Add-Type -Namespace WinInet -Name NativeMethods -MemberDefinition '[DllImport("wininet.dll", SetLastError=true)] public static extern bool InternetSetOption(IntPtr hInternet, int dwOption, IntPtr lpBuffer, int dwBufferLength);'
[WinInet.NativeMethods]::InternetSetOption([IntPtr]::Zero, 39, [IntPtr]::Zero, 0) | Out-Null
[WinInet.NativeMethods]::InternetSetOption([IntPtr]::Zero, 37, [IntPtr]::Zero, 0) | Out-Null
"#;
    let mut powershell = Command::new("powershell");
    hide_window(&mut powershell);
    let _ = powershell
        .args([
            "-NoProfile",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            script,
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
}

fn socks_address(profile: &ClientProfile) -> String {
    format!("{}:{}", profile.socks_host, profile.socks_port)
}

fn planned_steps(mode: &str) -> Vec<String> {
    if mode == "tun" {
        vec![
            "Start olcrtc SOCKS runtime".into(),
            "Start sing-box TUN bridge".into(),
            "Route system traffic through TUN".into(),
        ]
    } else {
        vec![
            "Start olcrtc SOCKS runtime".into(),
            "Expose local proxy".into(),
            "Use app/browser proxy settings".into(),
        ]
    }
}

fn default_language() -> String {
    "en".into()
}

fn default_proxy_auto() -> bool {
    true
}

fn time_stamp() -> String {
    let secs = unix_seconds() % 86_400;
    format!(
        "{:02}:{:02}:{:02}",
        secs / 3600,
        (secs / 60) % 60,
        secs % 60
    )
}

fn unix_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default()
}

fn lock_error<T>(_: std::sync::PoisonError<T>) -> String {
    "internal state lock failed".into()
}

pub fn run() {
    startup_cleanup();

    tauri::Builder::default()
        .manage(AppState::default())
        .setup(|app| {
            setup_tray(app)?;
            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .invoke_handler(tauri::generate_handler![
            get_profile,
            save_profile,
            dismiss_welcome,
            check_public_internet,
            get_servers,
            import_servers,
            select_server,
            delete_server,
            get_status,
            validate_uri,
            window_minimize,
            window_hide_to_tray,
            window_start_dragging,
            open_external,
            connect,
            disconnect,
        ])
        .run(tauri::generate_context!())
        .expect("error while running LibreRTC client");
}

fn setup_tray(app: &mut tauri::App) -> tauri::Result<()> {
    let show = MenuItem::with_id(app, "show", "Show LibreRTC", true, None::<&str>)?;
    let toggle = MenuItem::with_id(app, "toggle", "Connect", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show, &toggle, &quit])?;
    if let Some(state) = app.try_state::<AppState>() {
        if let Ok(mut slot) = state.tray_toggle.lock() {
            *slot = Some(toggle.clone());
        }
        if let Ok(status) = current_status(&state) {
            update_tray_toggle(&state, &status);
        }
    }
    TrayIconBuilder::new()
        .icon(tauri::include_image!("icons/icon.png"))
        .tooltip("LibreRTC")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_tray_icon_event(|tray, event| match event {
            TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            }
            | TrayIconEvent::DoubleClick {
                button: MouseButton::Left,
                ..
            } => show_main_window(tray.app_handle()),
            _ => {}
        })
        .on_menu_event(|app, event| match event.id().as_ref() {
            "show" => show_main_window(app),
            "toggle" => toggle_connection_from_tray(app),
            "quit" => app.exit(0),
            _ => {}
        })
        .build(app)?;
    Ok(())
}

fn toggle_connection_from_tray(app: &tauri::AppHandle) {
    let state = app.state::<AppState>();
    let running = state
        .store
        .lock()
        .map(|store| store.status.state == "connected" || store.status.state == "connecting")
        .unwrap_or(false);
    let result = if running {
        disconnect_inner(&state)
    } else {
        connect_inner(&state)
    };
    if let Ok(status) = result {
        update_tray_toggle(&state, &status);
    }
}

fn update_tray_toggle(state: &AppState, status: &ClientStatus) {
    let text = if status.state == "connected" || status.state == "connecting" {
        "Disconnect"
    } else {
        "Connect"
    };
    if let Ok(slot) = state.tray_toggle.lock() {
        if let Some(item) = slot.as_ref() {
            let _ = item.set_text(text);
            let _ = item.set_enabled(status.state != "connecting");
        }
    }
}

fn show_main_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
}

#[cfg(test)]
mod tests {
    use super::parse_runtime_uri;

    #[test]
    fn parses_olcrtc_uri_with_payload() {
        let target = parse_runtime_uri(
            "olcrtc://wbstream?vp8channel<vp8-batch=64&vp8-fps=60>@room-01#secret%client-01$Germany",
        )
        .expect("valid URI");

        assert_eq!(target.carrier, "wbstream");
        assert_eq!(target.transport, "vp8channel");
        assert_eq!(target.room_id, "room-01");
        assert_eq!(target.key, "secret");
        assert_eq!(target.client_id, "client-01");
        assert_eq!(target.name, "Germany");
        assert_eq!(target.payload.get("vp8-batch"), Some(&"64".to_string()));
        assert_eq!(target.payload.get("vp8-fps"), Some(&"60".to_string()));
    }
}
