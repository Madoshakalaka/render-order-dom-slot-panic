use std::net::SocketAddr;
use std::path::PathBuf;

use accept_language::intersection_ordered;
use anyhow::Result;
use axum::{
    Router,
    extract::Request,
    response::{Html, IntoResponse, Response},
};
use frontend::{DeviceInfo, ServerApp, ServerAppProps, SupportedLanguage};
use http::HeaderValue;
use stylist::manager::StyleManager;

async fn run() -> Result<()> {
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
    let registry = tracing_subscriber::registry();
    registry.with(tracing_subscriber::fmt::layer()).init();

    let port = std::env::var("PORT").map_or_else(|_| 80, |port| port.parse().unwrap());

    let app = Router::new().fallback(handler);

    let http_server = http_server(app, port);
    http_server.await;

    Ok(())
}

async fn http_server(app: Router, port: u16) {
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    axum_server::bind(addr).serve(app.into_make_service()).await.unwrap();
}

async fn handler(request: Request) -> Response {
    let path = request.uri().path();
    let headers = request.headers();

    // Serve static assets from filesystem
    if let Some(content) = serve_static_asset(path).await {
        return content;
    }

    // Extract headers for SSR
    let device_info = headers
        .get(http::header::USER_AGENT)
        .map(HeaderValue::as_ref)
        .map(std::str::from_utf8)
        .and_then(Result::ok)
        .map(DeviceInfo::parse_from);

    let presenting_language = headers
        .get(http::header::ACCEPT_LANGUAGE)
        .map(HeaderValue::as_ref)
        .map(std::str::from_utf8)
        .and_then(Result::ok)
        .and_then(|s| intersection_ordered(s, &SupportedLanguage::SUPPORTED_LANGUAGES).into_iter().next())
        .map(|s| SupportedLanguage::match_tag(&s).unwrap())
        .unwrap_or_default();

    // Render SSR
    let html_content = render_ssr(device_info, presenting_language).await;
    Html(html_content).into_response()
}

async fn serve_static_asset(path: &str) -> Option<Response> {
    let path = path.trim_start_matches('/');
    
    // Construct the file path
    let file_path = PathBuf::from("frontend/dist").join(path);
    
    // Try to read the file
    if let Ok(content) = tokio::fs::read(&file_path).await {
        let content_type = mime_guess::from_path(&file_path)
            .first_or_octet_stream()
            .as_ref()
            .to_string();
        
        return Some(([(http::header::CONTENT_TYPE, content_type)], content).into_response());
    }
    
    None
}

async fn render_ssr(device_info: Option<DeviceInfo>, presenting_language: SupportedLanguage) -> String {
    // Read the index.html template from filesystem
    let index_html = tokio::fs::read_to_string("frontend/dist/index.html")
        .await
        .unwrap_or_else(|_| String::from("<!DOCTYPE html><html><head></head><body></body></html>"));
    
    let (writer, reader) = stylist::manager::render_static();
    let (h_renderer, h_writer) = bounce::helmet::render_static();

    let body_s = {
        yew::ServerRenderer::<ServerApp>::with_props(move || {
            let manager = StyleManager::builder().writer(writer).build().expect("failed to create style manager.");
            ServerAppProps {
                h_writer,
                manager,
                device_info,
                presenting_language,
            }
        })
    }
    .render()
    .await;

    let rendered_helmet_tags = h_renderer.render().await;
    let mut rendered_head = String::new();
    for t in rendered_helmet_tags {
        t.write_static(&mut rendered_head).unwrap();
    }

    let data = reader.read_style_data();
    let mut style_s = String::new();
    data.write_static_markup(&mut style_s).expect("failed to read styles from style manager");

    // Split and inject into HTML template
    let head_split = index_html.split("</head>").collect::<Vec<_>>();
    let before_head = head_split[0];
    let after_head = head_split.get(1).unwrap_or(&"");

    let body_split = after_head.split("</body>").collect::<Vec<_>>();
    let before_body = body_split[0];
    let after_body = body_split.get(1).unwrap_or(&"");

    format!("{before_head}{style_s}{rendered_head}</head>{before_body}{body_s}</body>{after_body}")
}

#[tokio::main]
async fn main() {
    match run().await {
        Ok(_) => {}
        Err(e) => {
            tracing::error!("{}", e)
        }
    }
}