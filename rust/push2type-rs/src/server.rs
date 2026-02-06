use std::{
    sync::{Arc, Mutex},
    thread,
};

use crossbeam_channel::Sender;
use tiny_http::{Header, Method, Response, Server, StatusCode};

use crate::{
    app::AppEvent,
    config::AppConfig,
    tts::{SpeakRequest, TtsRequest},
};

pub fn spawn_server(
    config: Arc<Mutex<AppConfig>>,
    events: Sender<AppEvent>,
    tts_tx: Sender<TtsRequest>,
) {
    thread::spawn(move || {
        let port = config.lock().ok().map(|c| c.server_port).unwrap_or(7821);
        let addr = format!("127.0.0.1:{port}");
        let server = match Server::http(&addr) {
            Ok(s) => s,
            Err(e) => {
                let _ = events.send(AppEvent::Error(format!("server start failed: {e}")));
                return;
            }
        };
        let _ = events.send(AppEvent::ServerOnline(format!("http://{addr}/speak")));
        let _ = events.send(AppEvent::Info(format!(
            "endpoint online: http://{addr}/speak"
        )));

        for mut request in server.incoming_requests() {
            match (request.method(), request.url()) {
                (&Method::Get, "/health") => {
                    let body = r#"{"ok":true}"#;
                    let _ = request.respond(json_response(body, 200));
                }
                (&Method::Post, "/speak") => {
                    let mut body = String::new();
                    if request.as_reader().read_to_string(&mut body).is_err() {
                        let _ = request.respond(json_response(r#"{"error":"invalid body"}"#, 400));
                        continue;
                    }
                    match serde_json::from_str::<SpeakRequest>(&body) {
                        Ok(speak) => {
                            let _ = tts_tx.send(TtsRequest { speak });
                            let _ = request.respond(json_response(r#"{"accepted":true}"#, 202));
                        }
                        Err(e) => {
                            let _ =
                                events.send(AppEvent::Warning(format!("bad /speak request: {e}")));
                            let _ =
                                request.respond(json_response(r#"{"error":"invalid json"}"#, 400));
                        }
                    }
                }
                _ => {
                    let _ = request.respond(json_response(r#"{"error":"not found"}"#, 404));
                }
            }
        }
    });
}

fn json_response(body: &str, status: u16) -> Response<std::io::Cursor<Vec<u8>>> {
    let content_type =
        Header::from_bytes("Content-Type", "application/json").expect("static header");
    Response::from_data(body.as_bytes().to_vec())
        .with_status_code(StatusCode(status))
        .with_header(content_type)
}
