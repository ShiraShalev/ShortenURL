use axum::{extract::Path, response::Redirect, routing::get, Router};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;
use ShortenURL::generate_key;

type State = Arc<RwLock<(HashMap<String, String>, HashMap<String, String>)>>;

#[tokio::main]
async fn main() {
    let shared_state = Arc::new(RwLock::new((HashMap::new(), HashMap::new())));

    let app = Router::new()
        .route("/", get(|| async { "Welcome to URL Shortner!" }))
        .route("/404", get(|| async { "URL Not Found!" }))
        .route(
            "/shorten/{*url}",
            get({
                let shared_state = Arc::clone(&shared_state);
                move |path: Path<String>| shorten_url(path, shared_state)
            }),
        )
        .route(
            "/access/{url}",
            get({
                let shared_state = Arc::clone(&shared_state);
                move |path: Path<String>| redirect_url(path, shared_state)
            }),
        )
        .fallback(fallback_handler);

    let listener = match tokio::net::TcpListener::bind("0.0.0.0:3001").await {
        Ok(listener) => listener,
        Err(e) => {
            eprintln!("Failed to bind to port 3000: {}", e);
            return;
        }
    };
    match axum::serve(listener, app).await {
        Ok(_) => println!("Server running on port 3000"),
        Err(e) => eprintln!("Failed to start server: {}", e),
    }
}

async fn fallback_handler() -> Redirect {
    Redirect::temporary("/404")
}

async fn shorten_url(Path(path): Path<String>, state: State) -> String {
    let (url_to_key, key_to_url) = &mut *state.write().await;
    let uri = url_to_key
        .entry(path.clone())
        .or_insert_with(|| generate_unique_key(key_to_url));
    key_to_url.entry(uri.clone()).or_insert_with(|| path);

    format!("Your new URL: http://localhost:3000/access/{}", uri)
}

fn generate_unique_key(key_to_url: &HashMap<String, String>) -> String {
    loop {
        let key = generate_key();
        if !key_to_url.contains_key(&key) {
            return key;
        }
    }
}

async fn redirect_url(Path(path): Path<String>, state: State) -> Result<Redirect, Redirect> {
    let (_, ref key_to_url) = *state.read().await;
    match key_to_url.get(&path) {
        None => Err(fallback_handler().await),
        Some(long_url) => {
            println!("Redirecting from {} to {}", path, long_url);
            let go_to = if !long_url.starts_with("http://") && !long_url.starts_with("https://") {
                format!("http://{}", long_url)
            } else {
                long_url.clone()
            };
            print!("Redirecting to {}", go_to);
            Ok(Redirect::permanent(&go_to))
        }
    }
}
