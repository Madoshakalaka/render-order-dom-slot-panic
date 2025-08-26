use std::net::SocketAddr;
use std::path::PathBuf;
use anyhow::Result;
use axum::{
    Router,
    extract::Request,
    response::{Html, IntoResponse, Response},
};
use frontend::{App};
use yew::ServerRenderer;

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

    // Render SSR
    let html_content = render_ssr().await;
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

async fn render_ssr() -> String {
    // Read the index.html template from filesystem
    let index_html = tokio::fs::read_to_string("frontend/dist/index.html")
        .await
        .unwrap_or_else(|_| String::from("<!DOCTYPE html><html><head></head><body></body></html>"));
    

    let body_s = 
        ServerRenderer::<App>::new().render()
    .await;

    let body_split = index_html.split("</body>").collect::<Vec<_>>();
    let before_body = body_split[0];
    let after_body = body_split.get(1).unwrap_or(&"");

    format!("{before_body}{body_s}</body>{after_body}")
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
