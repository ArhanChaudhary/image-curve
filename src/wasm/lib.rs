use std::fmt;

// use renderer::DeserializeableOffscreenCanvas;
use serde::{
    de::{MapAccess, Visitor},
    Deserialize, Deserializer,
};
use wasm_bindgen::prelude::*;
use web_sys::console;

mod gilbert;
mod renderer;

#[wasm_bindgen(start)]
pub fn start() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

#[wasm_bindgen(js_name = handleMessage)]
pub fn handle_message(message: JsValue) {
    console::log_1(&"entry".into());
    let received_worker_message: ReceivedWorkerMessage =
        serde_wasm_bindgen::from_value(message).unwrap();
    ReceivedWorkerMessage::handle(received_worker_message);
}

enum ReceivedWorkerMessage {
    CanvasInit(CanvasInitMessage),
    Step,
}

#[derive(Deserialize)]
struct CanvasInitMessage {
    #[serde(rename = "pixelData")]
    image_buffer: Vec<u8>,
    width: usize,
    height: usize,
}

impl ReceivedWorkerMessage {
    pub fn handle(received_worker_message: Self) {
        match received_worker_message {
            Self::CanvasInit(canvas_init_message) => {
                renderer::canvas_init(canvas_init_message);
            }
            Self::Step => {
                renderer::step();
            }
        }
    }
}

struct ReceivedWorkerMessageVisitor;
impl<'de> Visitor<'de> for ReceivedWorkerMessageVisitor {
    type Value = ReceivedWorkerMessage;

    fn expecting(&self, _: &mut fmt::Formatter) -> fmt::Result {
        unreachable!()
    }

    fn visit_map<M>(self, mut map: M) -> Result<ReceivedWorkerMessage, M::Error>
    where
        M: MapAccess<'de>,
    {
        let action = map.next_entry::<String, String>()?.unwrap().1;
        Ok(match action.as_str() {
            "canvasInit" => ReceivedWorkerMessage::CanvasInit(CanvasInitMessage {
                image_buffer: map.next_entry::<String, _>()?.unwrap().1,
                width: map.next_entry::<String, _>()?.unwrap().1,
                height: map.next_entry::<String, _>()?.unwrap().1,
            }),
            "step" => ReceivedWorkerMessage::Step,
            _ => unreachable!(),
        })
    }
}

impl<'de> Deserialize<'de> for ReceivedWorkerMessage {
    fn deserialize<D>(deserializer: D) -> Result<ReceivedWorkerMessage, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(ReceivedWorkerMessageVisitor)
    }
}
