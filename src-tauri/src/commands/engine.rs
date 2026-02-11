use crate::midi_engine;
use crate::models::{EngineStatus, OscListenProtocol};
use crate::osc_engine;
use crate::router::{IncomingMessage, Router};
use crate::state::{AppState, EngineHandle};

use log::{info, warn};
use tauri::{AppHandle, Emitter, Manager, State};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

#[tauri::command]
pub async fn start_engine(state: State<'_, AppState>, app: AppHandle) -> Result<(), String> {
    // Check if already running
    {
        let engine = state.engine.lock().map_err(|e| e.to_string())?;
        if engine.is_some() {
            return Err("Engine is already running".to_string());
        }
    }

    let settings = {
        let s = state.settings.lock().map_err(|e| e.to_string())?;
        s.clone()
    };

    info!(
        "Starting engine — OSC listen :{} {:?}, send {}:{} {:?}, MIDI in: {:?}, out: {:?}",
        settings.osc_listen_port,
        settings.osc_listen_protocol,
        settings.osc_send_host,
        settings.osc_send_port,
        settings.osc_send_protocol,
        settings.midi_input_port_name,
        settings.midi_output_port_name,
    );

    let settings_arc = state.settings.clone();
    let mappings_arc = state.mappings.clone();

    let token = CancellationToken::new();
    let (tx, mut rx) = mpsc::unbounded_channel::<IncomingMessage>();

    // Open MIDI input
    let midi_input_conn = if let Some(ref port_name) = settings.midi_input_port_name {
        match midi_engine::open_input(port_name, tx.clone()) {
            Ok(conn) => Some(conn),
            Err(e) => {
                warn!("MIDI input warning: {}", e);
                let _ = app.emit(
                    "engine-status",
                    EngineStatus {
                        running: true,
                        error: Some(format!("MIDI input warning: {}", e)),
                    },
                );
                None
            }
        }
    } else {
        None
    };

    // Open MIDI output
    let midi_output_conn = if let Some(ref port_name) = settings.midi_output_port_name {
        match midi_engine::open_output(port_name) {
            Ok(conn) => Some(conn),
            Err(e) => {
                warn!("MIDI output warning: {}", e);
                let _ = app.emit(
                    "engine-status",
                    EngineStatus {
                        running: true,
                        error: Some(format!("MIDI output warning: {}", e)),
                    },
                );
                None
            }
        }
    } else {
        None
    };

    // Start OSC listeners
    match settings.osc_listen_protocol {
        OscListenProtocol::Udp | OscListenProtocol::Both => {
            osc_engine::start_udp_listener(settings.osc_listen_port, tx.clone(), token.clone())
                .await?;
        }
        _ => {}
    }
    match settings.osc_listen_protocol {
        OscListenProtocol::Tcp | OscListenProtocol::Both => {
            osc_engine::start_tcp_listener(settings.osc_listen_port, tx.clone(), token.clone())
                .await?;
        }
        _ => {}
    }

    // Spawn router task
    let router = Router::new(mappings_arc, app.clone());
    let midi_out_for_router = midi_output_conn;
    let router_token = token.clone();

    let rt = tokio::runtime::Handle::current();
    tokio::spawn(async move {
        loop {
            tokio::select! {
                _ = router_token.cancelled() => break,
                msg = rx.recv() => {
                    match msg {
                        Some(incoming) => {
                            let actions = router.route(&incoming);
                            for action in &actions {
                                osc_engine::dispatch_output(
                                    action,
                                    &settings_arc,
                                    &midi_out_for_router,
                                    &rt,
                                );
                            }
                        }
                        None => break,
                    }
                }
            }
        }
    });

    // Spawn MIDI hot-plug detection
    let hotplug_token = token.clone();
    let hotplug_app = app.clone();
    let hotplug_settings = state.settings.clone();
    tokio::spawn(async move {
        let (mut last_inputs, mut last_outputs) = midi_engine::enumerate_ports_hash();

        loop {
            tokio::select! {
                _ = hotplug_token.cancelled() => break,
                _ = tokio::time::sleep(std::time::Duration::from_secs(2)) => {
                    let (inputs, outputs) = midi_engine::enumerate_ports_hash();
                    if inputs != last_inputs || outputs != last_outputs {
                        info!("MIDI device change detected");
                        last_inputs = inputs.clone();
                        last_outputs = outputs.clone();
                        let _ = hotplug_app.emit("midi-devices-changed", ());

                        // Check if active port disappeared
                        let current_settings = match hotplug_settings.lock() {
                            Ok(guard) => guard.clone(),
                            Err(e) => {
                                warn!("Settings mutex poisoned in hotplug loop: {}", e);
                                continue;
                            }
                        };
                        let mut disconnected = false;
                        if let Some(ref name) = current_settings.midi_input_port_name {
                            if !inputs.contains(name) {
                                warn!("MIDI input disconnected: {}", name);
                                let _ = hotplug_app.emit("engine-status", EngineStatus {
                                    running: false,
                                    error: Some(format!("MIDI input disconnected: {}", name)),
                                });
                                disconnected = true;
                            }
                        }
                        if let Some(ref name) = current_settings.midi_output_port_name {
                            if !outputs.contains(name) {
                                warn!("MIDI output disconnected: {}", name);
                                let _ = hotplug_app.emit("engine-status", EngineStatus {
                                    running: false,
                                    error: Some(format!("MIDI output disconnected: {}", name)),
                                });
                                disconnected = true;
                            }
                        }
                        if disconnected {
                            // Use app handle to access state and stop engine
                            let state: State<AppState> = hotplug_app.state();
                            if let Ok(mut eng) = state.engine.lock() {
                                if let Some(handle) = eng.take() {
                                    handle.cancel_token.cancel();
                                }
                            }
                            break;
                        }
                    }
                }
            }
        }
    });

    // Store handle
    {
        let mut engine = state.engine.lock().map_err(|e| e.to_string())?;
        *engine = Some(EngineHandle {
            cancel_token: token,
            _midi_input: midi_input_conn,
        });
    }

    let _ = app.emit(
        "engine-status",
        EngineStatus {
            running: true,
            error: None,
        },
    );

    Ok(())
}

#[tauri::command]
pub fn stop_engine(state: State<AppState>, app: AppHandle) -> Result<(), String> {
    info!("Stopping engine");
    let mut engine = state.engine.lock().map_err(|e| e.to_string())?;
    if let Some(handle) = engine.take() {
        handle.cancel_token.cancel();
    }
    let _ = app.emit(
        "engine-status",
        EngineStatus {
            running: false,
            error: None,
        },
    );
    Ok(())
}

#[tauri::command]
pub fn get_engine_status(state: State<AppState>) -> Result<EngineStatus, String> {
    let engine = state.engine.lock().map_err(|e| e.to_string())?;
    Ok(EngineStatus {
        running: engine.is_some(),
        error: None,
    })
}
