use crate::models::{Mapping, Settings};
use midir::MidiInputConnection;
use std::sync::{Arc, Mutex};
use tokio_util::sync::CancellationToken;

pub struct EngineHandle {
    pub cancel_token: CancellationToken,
    pub _midi_input: Option<MidiInputConnection<()>>,
}

// SAFETY: MidiInputConnection<()> is not Send because the underlying platform MIDI handle
// types (e.g. CoreMIDI's MIDIPortRef on macOS) are raw pointers. However, this is safe because:
// 1. Platform MIDI handles (CoreMIDI, ALSA, WinMM) are thread-safe in practice — they are
//    opaque handles managed by the OS and do not carry thread-affine mutable state.
// 2. We only store the connection here to keep it alive and drop it on engine stop.
//    No cross-thread method calls are made on the connection itself.
// 3. The callback closure captured by midir only captures an mpsc::UnboundedSender<IncomingMessage>,
//    which is Send + Sync.
unsafe impl Send for EngineHandle {}

pub struct AppState {
    pub settings: Arc<Mutex<Settings>>,
    pub mappings: Arc<Mutex<Vec<Mapping>>>,
    pub engine: Mutex<Option<EngineHandle>>,
}
