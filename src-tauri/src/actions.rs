use std::process::Command;

use serde::Deserialize;
use tauri::{AppHandle, Manager, Runtime};
use tauri_plugin_opener::OpenerExt;

pub const SYSTEM_PROMPT: &str = r#"Você é um assistente de voz. O usuário fala comandos e você escolhe UMA das ações disponíveis:

- "abrir_navegador": abrir o navegador sem pesquisar nada.
- "pesquisar_google": pesquisar no Google. "parametro" = o termo da pesquisa.
- "pesquisar_youtube": pesquisar no YouTube. "parametro" = o termo da pesquisa.
- "abrir_site": abrir um site. "parametro" = o endereço (ex.: "github.com").
- "abrir_explorador": abrir o explorador de arquivos.
- "abrir_pasta": abrir uma pasta. "parametro" = "downloads", "documentos", "area_de_trabalho" ou "imagens".
- "abrir_calculadora": abrir a calculadora.
- "responder": qualquer outra coisa (perguntas, conversa, pedidos que não correspondem a uma ação acima).

Se o usuário pedir só para abrir o navegador, use "abrir_navegador". Se pedir para pesquisar algo, use "pesquisar_google" com o termo.

Responda SOMENTE com um objeto JSON, sem texto extra e sem markdown, neste formato:
{"acao": "<nome da ação>", "parametro": "<valor ou vazio>", "resposta": "<frase curta em português>"}

Em "resposta", confirme a ação em poucas palavras (ex.: "Pesquisando receita de bolo.") ou, para "responder", responda ao usuário de forma curta e direta."#;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    OpenBrowser,
    SearchGoogle(String),
    SearchYoutube(String),
    OpenSite(String),
    OpenFileExplorer,
    OpenFolder(String),
    OpenCalculator,
    Reply,
}

impl Action {
    pub fn label(&self) -> String {
        match self {
            Action::OpenBrowser => "abrir navegador".into(),
            Action::SearchGoogle(query) => format!("pesquisar no Google: {query}"),
            Action::SearchYoutube(query) => format!("pesquisar no YouTube: {query}"),
            Action::OpenSite(site) => format!("abrir site: {site}"),
            Action::OpenFileExplorer => "abrir explorador".into(),
            Action::OpenFolder(folder) => format!("abrir pasta: {folder}"),
            Action::OpenCalculator => "abrir calculadora".into(),
            Action::Reply => "responder".into(),
        }
    }

    fn from_parts(name: &str, parameter: &str) -> Self {
        let parameter = parameter.trim().to_string();
        match name {
            "abrir_navegador" => Action::OpenBrowser,
            "pesquisar_google" if parameter.is_empty() => Action::OpenBrowser,
            "pesquisar_google" => Action::SearchGoogle(parameter),
            "pesquisar_youtube" if parameter.is_empty() => Action::OpenSite("youtube.com".into()),
            "pesquisar_youtube" => Action::SearchYoutube(parameter),
            "abrir_site" if !parameter.is_empty() => Action::OpenSite(parameter),
            "abrir_explorador" => Action::OpenFileExplorer,
            "abrir_pasta" if !parameter.is_empty() => Action::OpenFolder(parameter),
            "abrir_calculadora" => Action::OpenCalculator,
            _ => Action::Reply,
        }
    }
}

#[derive(Debug, Deserialize)]
struct RawDecision {
    #[serde(rename = "acao", default)]
    action: String,
    #[serde(rename = "parametro", default)]
    parameter: String,
    #[serde(rename = "resposta", default)]
    reply: String,
}

#[derive(Debug)]
pub struct Decision {
    pub action: Action,
    pub reply: String,
}

pub fn parse(output: &str) -> Decision {
    let trimmed = output
        .trim()
        .trim_start_matches("```json")
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim();

    match serde_json::from_str::<RawDecision>(trimmed) {
        Ok(raw) => Decision {
            action: Action::from_parts(raw.action.trim(), &raw.parameter),
            reply: raw.reply.trim().to_string(),
        },
        Err(_) => Decision {
            action: Action::Reply,
            reply: output.trim().to_string(),
        },
    }
}

fn open_url<R: Runtime>(app: &AppHandle<R>, url: &str) -> Result<(), String> {
    app.opener()
        .open_url(url, None::<&str>)
        .map_err(|e| e.to_string())
}

fn search_url(base: &str, key: &str, query: &str) -> Result<String, String> {
    reqwest::Url::parse_with_params(base, [(key, query)])
        .map(|url| url.to_string())
        .map_err(|e| e.to_string())
}

fn normalize_site(site: &str) -> Result<String, String> {
    let site = site.trim();
    let candidate = if site.contains("://") {
        site.to_string()
    } else {
        format!("https://{site}")
    };
    let url = reqwest::Url::parse(&candidate).map_err(|e| e.to_string())?;
    match url.scheme() {
        "http" | "https" if url.host_str().is_some() => Ok(url.to_string()),
        _ => Err(format!("Endereço não permitido: {site}")),
    }
}

fn open_folder<R: Runtime>(app: &AppHandle<R>, folder: &str) -> Result<(), String> {
    let paths = app.path();
    let dir = match folder {
        "downloads" => paths.download_dir(),
        "documentos" => paths.document_dir(),
        "area_de_trabalho" => paths.desktop_dir(),
        "imagens" => paths.picture_dir(),
        other => return Err(format!("Pasta desconhecida: {other}")),
    }
    .map_err(|e| e.to_string())?;

    app.opener()
        .open_path(dir.to_string_lossy().to_string(), None::<&str>)
        .map_err(|e| e.to_string())
}

fn open_calculator() -> Result<(), String> {
    #[cfg(target_os = "macos")]
    let mut command = {
        let mut command = Command::new("open");
        command.args(["-a", "Calculator"]);
        command
    };
    #[cfg(target_os = "windows")]
    let mut command = Command::new("calc");
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    let mut command = Command::new("gnome-calculator");

    command.spawn().map(|_| ()).map_err(|e| e.to_string())
}

pub fn execute<R: Runtime>(app: &AppHandle<R>, action: &Action) -> Result<(), String> {
    match action {
        Action::OpenBrowser => open_url(app, "https://www.google.com"),
        Action::SearchGoogle(query) => open_url(
            app,
            &search_url("https://www.google.com/search", "q", query)?,
        ),
        Action::SearchYoutube(query) => open_url(
            app,
            &search_url("https://www.youtube.com/results", "search_query", query)?,
        ),
        Action::OpenSite(site) => open_url(app, &normalize_site(site)?),
        Action::OpenFileExplorer => {
            let home = app.path().home_dir().map_err(|e| e.to_string())?;
            app.opener()
                .open_path(home.to_string_lossy().to_string(), None::<&str>)
                .map_err(|e| e.to_string())
        }
        Action::OpenFolder(folder) => open_folder(app, folder),
        Action::OpenCalculator => open_calculator(),
        Action::Reply => Ok(()),
    }
}
