use axum::response::sse as ax_sse;
use axum::{
    extract::State,
    http::{header, StatusCode, Uri},
    response::{IntoResponse, sse::{Event, Sse}},
    routing::{get, post},
    Json, Router,
};
use rust_embed::RustEmbed;
use serde::{Serialize, Deserialize};
use std::{sync::{Arc, RwLock}, time::Duration, collections::HashSet};
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::StreamExt;
use regex::Regex;
use std::convert::Infallible;

#[derive(RustEmbed)]
#[folder = "ui_dist"] 
struct Assets;

#[derive(Clone, Serialize, Debug)]
struct Source { url: String, topic: String, description: String }

#[derive(Clone, Serialize, Debug)]
#[serde(tag = "type", content = "payload")]
enum AppEvent { Log { time: String, msg: String, level: String }, Source(Source), Status { running: bool, count: u32, target: u32, has_custom_topics: bool } }

#[derive(Deserialize)]
struct ConfigReq { target: u32 }

#[derive(Deserialize)]
struct TopicsReq { topics: Vec<String> }

struct AppState {
    tx: broadcast::Sender<AppEvent>,
    running: Arc<RwLock<bool>>,
    data: Arc<RwLock<Vec<Source>>>,
    history: Arc<RwLock<HashSet<String>>>,
    target: Arc<RwLock<u32>>,
    custom_topics: Arc<RwLock<Vec<String>>>,
    openai_key: String,
}

#[tokio::main]
async fn main() {
    let openai_key = std::env::var("OPENAI_API_KEY").unwrap_or_default();
    let (tx, _) = broadcast::channel(200);
    let state = Arc::new(AppState {
        tx: tx.clone(),
        running: Arc::new(RwLock::new(false)),
        data: Arc::new(RwLock::new(Vec::new())),
        history: Arc::new(RwLock::new(HashSet::new())),
        target: Arc::new(RwLock::new(10)),
        custom_topics: Arc::new(RwLock::new(Vec::new())),
        openai_key,
    });

    let app = Router::new()
        .route("/api/start", post(start_agent))
        .route("/api/stop", post(stop_agent))
        .route("/api/config", post(update_config))
        .route("/api/topics", post(update_topics))
        .route("/api/sse", get(sse_handler))
        .route("/api/export", get(export_handler))
        .fallback(static_handler)
        .with_state(state);

    println!("✅ SERVIDOR ACTIVO EN PUERTO 3000");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn static_handler(uri: Uri) -> impl IntoResponse {
    let path = uri.path().trim_start_matches('/').to_string();
    let file_path = if path.is_empty() || path.ends_with('/') { "index.html" } else { &path };
    match Assets::get(file_path) {
        Some(content) => {
            let mime = mime_guess::from_path(file_path).first_or_octet_stream();
            ([(header::CONTENT_TYPE, mime.as_ref())], content.data).into_response()
        }
        None => {
            let index = Assets::get("index.html").expect("UI error");
            ([(header::CONTENT_TYPE, "text/html")], index.data).into_response()
        }
    }
}

async fn update_config(State(state): State<Arc<AppState>>, Json(req): Json<ConfigReq>) -> impl IntoResponse {
    let mut t = state.target.write().unwrap();
    *t = req.target;
    StatusCode::OK
}

async fn update_topics(State(state): State<Arc<AppState>>, Json(req): Json<TopicsReq>) -> impl IntoResponse {
    let mut t = state.custom_topics.write().unwrap();
    *t = req.topics.clone();
    let _ = state.tx.send(AppEvent::Log { time: now(), msg: format!("📂 Cargadas {} temáticas personalizadas.", req.topics.len()), level: "SUCCESS".into() });
    StatusCode::OK
}

async fn start_agent(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    { let mut r = state.running.write().unwrap(); if *r { return StatusCode::OK; } *r = true; }
    state.data.write().unwrap().clear();
    state.history.write().unwrap().clear();
    let s = state.clone();
    tokio::spawn(async move { hunter_engine(s).await; });
    StatusCode::OK
}

async fn stop_agent(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    { let mut r = state.running.write().unwrap(); *r = false; }
    StatusCode::OK
}

async fn export_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let data = state.data.read().unwrap();
    let mut csv = String::from("URL,TEMA,DESCRIPCION\n");
    for d in data.iter() {
        csv.push_str(&format!("\"{}\",\"{}\",\"{}\"\n", d.url, d.topic, d.description.replace("\"", "'")));
    }
    ([(header::CONTENT_TYPE, "text/csv"), (header::CONTENT_DISPOSITION, "attachment; filename=\"data.csv\"")], csv)
}

async fn sse_handler(State(state): State<Arc<AppState>>) -> Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>> {
    let rx = state.tx.subscribe();
    let stream = BroadcastStream::new(rx).map(|msg| {
        let json = serde_json::to_string(&msg.unwrap()).unwrap();
        Ok(Event::default().data(json))
    });
    Sse::new(stream).keep_alive(ax_sse::KeepAlive::new())
}

async fn hunter_engine(state: Arc<AppState>) {
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .timeout(Duration::from_secs(12)).build().unwrap();

    push_log(&state, "INFO", "🚀 Protocolo Hunter activado. Iniciando búsqueda...");

    // 1. FUENTES SEMILLA INMEDIATAS
    let seeds = vec![
        ("https://www.kaggle.com/datasets", "CIENCIA DE DATOS", "Repositorio masivo para Machine Learning."),
        ("https://data.worldbank.org/", "ECONOMÍA", "Banco Mundial Open Data."),
        ("https://archive.ics.uci.edu/", "ACADEMIA", "UCI Machine Learning Library."),
        ("https://github.com/awesomedata/awesome-public-datasets", "GENERAL", "Indice maestro de datasets públicos.")
    ];

    for (url, topic, desc) in seeds {
        if !is_running(&state) || is_target_met(&state) { break; }
        add_source(&state, url.to_string(), topic.to_string(), desc.to_string()).await;
    }

    // 2. CAZA EXPANSIVA (CSV o DEEP SCAN)
    let master_repos = vec![
        "https://raw.githubusercontent.com/awesomedata/awesome-public-datasets/master/README.md",
        "https://raw.githubusercontent.com/datasets/awesome-data/master/README.md",
        "https://raw.githubusercontent.com/onurakpolat/awesome-bigdata/master/README.md"
    ];

    let mut r_idx = 0;
    let mut q_idx = 0;

    while is_running(&state) && !is_target_met(&state) {
        let mut links = Vec::new();
        let custom_topics = state.custom_topics.read().unwrap().clone();

        if !custom_topics.is_empty() {
            let t = &custom_topics[q_idx % custom_topics.len()];
            push_log(&state, "INFO", &format!("🔎 Buscando temática CSV: {}", t));
            let url = format!("https://html.duckduckgo.com/html/?q={}", urlencoding::encode(&format!("datasets {} csv open data", t)));
            if let Ok(res) = client.get(url).send().await {
                if let Ok(body) = res.text().await {
                    let re = Regex::new(r"https?://[^\s\)\]]+").unwrap();
                    links = re.find_iter(&body).map(|m| m.as_str().to_string()).collect();
                }
            }
            q_idx += 1;
        } else if r_idx < master_repos.len() {
            let url = master_repos[r_idx];
            push_log(&state, "WARN", &format!("🚀 Escaneando repositorio maestro #{}...", r_idx + 1));
            if let Ok(res) = client.get(url).send().await {
                if let Ok(body) = res.text().await {
                    let re = Regex::new(r"https?://[^\s\)\]]+").unwrap();
                    links = re.find_iter(&body).map(|m| m.as_str().to_string()).collect();
                }
            }
            r_idx += 1;
        } else {
            // Si todo falla, búsqueda genérica
            push_log(&state, "INFO", "🔎 Generando búsqueda web genérica...");
            let url = "https://html.duckduckgo.com/html/?q=open+data+portal+directory+csv";
            if let Ok(res) = client.get(url).send().await {
                if let Ok(body) = res.text().await {
                    let re = Regex::new(r"https?://[^\s\)\]]+").unwrap();
                    links = re.find_iter(&body).map(|m| m.as_str().to_string()).collect();
                }
            }
        }

        for url in links {
            if !is_running(&state) || is_target_met(&state) { break; }
            if url.contains("duckduckgo") || url.contains("github.com") || url.len() < 20 { continue; }
            
            if !state.history.read().unwrap().contains(&url) {
                state.history.write().unwrap().insert(url.clone());
                push_log(&state, "INFO", &format!("🧠 Analizando: {}", url));
                let source = analyze_with_ai(&state, &url).await;
                add_source(&state, source.url, source.topic, source.description).await;
            }
        }
        tokio::time::sleep(Duration::from_secs(2)).await;
    }
    push_log(&state, "SUCCESS", "✅ Caza finalizada.");
    let target = *state.target.read().unwrap();
    let count = state.data.read().unwrap().len() as u32;
    let _ = state.tx.send(AppEvent::Status { running: false, count, target, has_custom_topics: !state.custom_topics.read().unwrap().is_empty() });
    *state.running.write().unwrap() = false;
}

async fn add_source(state: &Arc<AppState>, url: String, topic: String, description: String) {
    let source = Source { url, topic, description };
    state.data.write().unwrap().push(source.clone());
    let _ = state.tx.send(AppEvent::Source(source));
    let count = state.data.read().unwrap().len() as u32;
    let target = *state.target.read().unwrap();
    let has_custom = !state.custom_topics.read().unwrap().is_empty();
    let _ = state.tx.send(AppEvent::Status { running: true, count, target, has_custom_topics: has_custom });
    tokio::time::sleep(Duration::from_millis(600)).await;
}

fn is_running(state: &Arc<AppState>) -> bool { *state.running.read().unwrap() }
fn is_target_met(state: &Arc<AppState>) -> bool { state.data.read().unwrap().len() as u32 >= *state.target.read().unwrap() }

async fn analyze_with_ai(state: &Arc<AppState>, url: &str) -> Source {
    if state.openai_key.is_empty() || state.openai_key.len() < 10 {
        let topic = if url.contains("gov") { "GOBIERNO" } else if url.contains("edu") { "ACADEMIA" } else { "OPEN DATA" };
        return Source { url: url.into(), topic: topic.to_string(), description: "Recurso detectado mediante rastreo profundo.".to_string() };
    }
    let client = reqwest::Client::new();
    let payload = serde_json::json!({
        "model": "gpt-4o-mini",
        "messages": [{"role": "user", "content": format!("Analiza el link: {}. Explica en español que datos ofrece en 15 palabras.", url)}],
        "max_tokens": 60
    });
    if let Ok(res) = client.post("https://api.openai.com/v1/chat/completions").bearer_auth(&state.openai_key).json(&payload).send().await {
        if let Ok(json) = res.json::<serde_json::Value>().await {
            let desc = json["choices"][0]["message"]["content"].as_str().unwrap_or("Dataset verificado.");
            let topic = if url.contains("gov") { "GOBIERNO" } else { "GENERAL" };
            return Source { url: url.into(), topic: topic.to_string(), description: desc.trim().replace("\"", "").into() };
        }
    }
    Source { url: url.into(), topic: "WEB".to_string(), description: "Link de datos identificado.".to_string() }
}

fn push_log(s: &Arc<AppState>, level: &str, msg: &str) {
    let _ = s.tx.send(AppEvent::Log { time: now(), msg: msg.into(), level: level.into() });
}
fn now() -> String { chrono::Local::now().format("%H:%M:%S").to_string() }
