//! Serves moodboard images to the web view as `hodie-image://localhost/<moodboard id>/<file>`
//! (on Windows `http://hodie-image.localhost/…`, see `convertFileSrc`). Only
//! files a moodboard of the open vault refers to are served.

use std::thread;

use tauri::http::{Request, Response, StatusCode, header};
use tauri::{Manager, Runtime, UriSchemeContext, UriSchemeResponder};

use crate::library::Library;
use crate::vault::image::content_type;

pub const SCHEME: &str = "hodie-image";

pub fn handle<R: Runtime>(
    context: UriSchemeContext<'_, R>,
    request: Request<Vec<u8>>,
    responder: UriSchemeResponder,
) {
    let app = context.app_handle().clone();
    let path = request.uri().path().to_owned();
    // Reading a large photo must not hold up the web view.
    thread::spawn(move || responder.respond(respond(&app, &path)));
}

fn respond<R: Runtime>(app: &tauri::AppHandle<R>, path: &str) -> Response<Vec<u8>> {
    let not_found = || {
        Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Vec::new())
            .unwrap_or_default()
    };
    let Some((moodboard_id, file)) = parse_path(path) else {
        return not_found();
    };
    let Some(library) = app.try_state::<Library>() else {
        return not_found();
    };
    let Ok(full) = library.with(|store| store.image_path(&moodboard_id, &file)) else {
        return not_found();
    };
    let (Some(kind), Ok(bytes)) = (content_type(&file), std::fs::read(full)) else {
        return not_found();
    };
    Response::builder()
        .header(header::CONTENT_TYPE, kind)
        // Image files never change; a changed image gets a new name.
        .header(header::CACHE_CONTROL, "max-age=31536000, immutable")
        .body(bytes)
        .unwrap_or_else(|_| not_found())
}

/// Moodboard id and file name from the request path. `convertFileSrc`
/// encodes the whole path, so the separating slash arrives as `%2F`.
fn parse_path(path: &str) -> Option<(String, String)> {
    let decoded = percent_decode(path.trim_start_matches('/'))?;
    let (moodboard_id, file) = decoded.split_once('/')?;
    (!moodboard_id.is_empty() && !file.is_empty() && !file.contains(['/', '\\']))
        .then(|| (moodboard_id.to_owned(), file.to_owned()))
}

fn percent_decode(text: &str) -> Option<String> {
    let bytes = text.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' {
            let hex = std::str::from_utf8(bytes.get(i + 1..i + 3)?).ok()?;
            out.push(u8::from_str_radix(hex, 16).ok()?);
            i += 3;
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }
    String::from_utf8(out).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paths_name_a_moodboard_and_a_file() {
        let pair = |a: &str, b: &str| Some((a.to_owned(), b.to_owned()));
        assert_eq!(parse_path("/01JB%2F01JC.png"), pair("01JB", "01JC.png"));
        assert_eq!(parse_path("/01JB/01JC.png"), pair("01JB", "01JC.png"));
        assert_eq!(parse_path("/01JB%2F..%2Fboard.json"), None);
        assert_eq!(parse_path("/01JB%2Fa%5Cb.png"), None);
        assert_eq!(parse_path("/01JB"), None);
        assert_eq!(parse_path("/%2Fa.png"), None);
        assert_eq!(parse_path("/01JB%2Fa%2"), None);
    }
}
