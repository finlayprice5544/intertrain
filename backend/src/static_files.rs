// Static frontend serving + SPA/404 fallback (used as the Axum fallback handler).
use crate::state::AppState;
use axum::{
    body::Body,
    extract::State,
    http::{header, StatusCode, Uri},
    response::{IntoResponse, Response},
};
use std::path::{Component, Path, PathBuf};

pub async fn fallback(State(st): State<AppState>, uri: Uri) -> Response {
    let path = uri.path();

    if path.starts_with("/api/") {
        return (
            StatusCode::NOT_FOUND,
            [(header::CONTENT_TYPE, "application/json")],
            format!("{{\"error\":\"not_found\",\"path\":\"{path}\"}}"),
        )
            .into_response();
    }

    let root = Path::new(&st.config.public_dir);

    // 1) exact file
    if let Some(resp) = serve_file(root, path).await {
        return resp;
    }
    // 2) directory-style page: <path>/index.html
    let dir_index = format!("{}/index.html", path.trim_end_matches('/'));
    if let Some(resp) = serve_file(root, &dir_index).await {
        return resp;
    }
    // 3) SPA fallback -> entry
    if st.spa {
        if let Some(resp) = serve_file(root, "/index.html").await {
            return resp;
        }
    }
    (StatusCode::NOT_FOUND, "404 Not Found").into_response()
}

/// Read a file safely under `root` (rejects traversal); guess content-type.
async fn serve_file(root: &Path, req_path: &str) -> Option<Response> {
    let rel = req_path.trim_start_matches('/');
    let mut full: PathBuf = root.to_path_buf();
    for comp in Path::new(rel).components() {
        match comp {
            Component::Normal(c) => full.push(c),
            Component::CurDir => {}
            _ => return None, // ParentDir / RootDir / Prefix -> reject
        }
    }
    let data = tokio::fs::read(&full).await.ok()?;
    let mime = mime_guess::from_path(&full).first_or_octet_stream();
    let mut builder = Response::builder().header(header::CONTENT_TYPE, mime.as_ref());
    if full.components().any(|c| c.as_os_str() == "assets") {
        builder = builder.header(header::CACHE_CONTROL, "public, max-age=31536000, immutable");
    }
    Some(builder.body(Body::from(data)).unwrap().into_response())
}
