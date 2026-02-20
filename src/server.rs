use std::{
    thread::{self, JoinHandle},
    time::Duration,
};

use crossbeam_channel::{Sender, unbounded};
use tiny_http::{Header, Method, Response, Server, StatusCode};

use crate::{
    app::AppEvent,
    tts::{SpeakRequest, TtsRequest},
};

#[derive(Clone)]
pub struct ServerControl {
    cmd_tx: Sender<ServerCommand>,
}

impl ServerControl {
    pub fn set_enabled(&self, enabled: bool) {
        let _ = self.cmd_tx.send(ServerCommand::SetEnabled(enabled));
    }

    pub fn set_port(&self, port: u16) {
        let _ = self.cmd_tx.send(ServerCommand::SetPort(port));
    }
}

enum ServerCommand {
    SetEnabled(bool),
    SetPort(u16),
}

struct RunningServer {
    port: u16,
    stop_tx: Sender<()>,
    join: JoinHandle<()>,
}

pub fn spawn_server_controller(
    initial_enabled: bool,
    initial_port: u16,
    events: Sender<AppEvent>,
    tts_tx: Sender<TtsRequest>,
) -> ServerControl {
    let (cmd_tx, cmd_rx) = unbounded::<ServerCommand>();
    let control = ServerControl { cmd_tx };
    thread::spawn(move || {
        let mut enabled = initial_enabled;
        let mut port = initial_port;
        let mut running = None;
        reconcile_server_state(enabled, port, &mut running, &events, &tts_tx);

        while let Ok(cmd) = cmd_rx.recv() {
            match cmd {
                ServerCommand::SetEnabled(next) => enabled = next,
                ServerCommand::SetPort(next) => port = next,
            }
            reconcile_server_state(enabled, port, &mut running, &events, &tts_tx);
        }

        stop_server(&mut running);
    });
    control
}

fn reconcile_server_state(
    enabled: bool,
    port: u16,
    running: &mut Option<RunningServer>,
    events: &Sender<AppEvent>,
    tts_tx: &Sender<TtsRequest>,
) {
    if !enabled {
        stop_server(running);
        return;
    }

    let needs_restart = match running {
        Some(active) => active.port != port,
        None => true,
    };
    if !needs_restart {
        return;
    }

    stop_server(running);
    *running = start_server(port, events, tts_tx);
}

fn stop_server(running: &mut Option<RunningServer>) {
    if let Some(active) = running.take() {
        let _ = active.stop_tx.send(());
        let _ = active.join.join();
    }
}

fn start_server(
    port: u16,
    events: &Sender<AppEvent>,
    tts_tx: &Sender<TtsRequest>,
) -> Option<RunningServer> {
    let addr = format!("127.0.0.1:{port}");
    let server = match Server::http(&addr) {
        Ok(s) => s,
        Err(e) => {
            let _ = events.send(AppEvent::Error(format!("server start failed: {e}")));
            let _ = events.send(AppEvent::ServerOffline);
            return None;
        }
    };
    let endpoint = format!("http://{addr}/speak");
    let events_clone = events.clone();
    let tts_tx_clone = tts_tx.clone();
    let (stop_tx, stop_rx) = unbounded::<()>();

    let join = thread::spawn(move || {
        let _ = events_clone.send(AppEvent::ServerOnline(endpoint.clone()));
        let _ = events_clone.send(AppEvent::Info(format!("endpoint online: {endpoint}")));

        loop {
            if stop_rx.try_recv().is_ok() {
                break;
            }
            let req = match server.recv_timeout(Duration::from_millis(200)) {
                Ok(r) => r,
                Err(e) => {
                    let _ = events_clone.send(AppEvent::Error(format!("server recv failed: {e}")));
                    break;
                }
            };
            let Some(mut request) = req else {
                continue;
            };
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
                            let _ = tts_tx_clone.send(TtsRequest { speak });
                            let _ = request.respond(json_response(r#"{"accepted":true}"#, 202));
                        }
                        Err(e) => {
                            let _ = events_clone
                                .send(AppEvent::Warning(format!("bad /speak request: {e}")));
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
        let _ = events_clone.send(AppEvent::ServerOffline);
        let _ = events_clone.send(AppEvent::Info("endpoint offline".to_string()));
    });

    Some(RunningServer {
        port,
        stop_tx,
        join,
    })
}

fn json_response(body: &str, status: u16) -> Response<std::io::Cursor<Vec<u8>>> {
    let content_type =
        Header::from_bytes("Content-Type", "application/json").expect("static header");
    Response::from_data(body.as_bytes().to_vec())
        .with_status_code(StatusCode(status))
        .with_header(content_type)
}
