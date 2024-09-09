use handlers::RequestAnimationFrameHandle;
use js_sys::Array;
use renderer::ImageDimensions;
use serde::Serialize;
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use wasm_bindgen::prelude::*;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, HtmlInputElement, Worker};

mod handlers;
mod paths;
mod renderer;
mod utils;
mod worker;

#[wasm_bindgen(start)]
fn start() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

struct GlobalState {
    ctx: CanvasRenderingContext2d,
    upload_input: HtmlInputElement,
    start_input: HtmlInputElement,
    worker: Worker,
    image_dimensions: RefCell<ImageDimensions>,
    raf_handle: RefCell<Option<RequestAnimationFrameHandle>>,
    path_len: Cell<Option<u32>>,
    change_speed_input: HtmlInputElement,
    change_step_input: HtmlInputElement,
}

struct LocalState {
    step_input: HtmlInputElement,
    stop_input: HtmlInputElement,
}

#[wasm_bindgen(js_name = runMain)]
pub fn run_main(worker: Worker) {
    let worker_message = Array::new();
    worker_message.push(&wasm_bindgen::module());
    worker_message.push(&wasm_bindgen::memory());
    worker.post_message(&worker_message).unwrap();

    let document = web_sys::window().unwrap().document().unwrap();

    let ctx = utils::get_element_by_id::<HtmlCanvasElement>(&document, "canvas")
        .get_context_with_context_options(
            "2d",
            &serde_wasm_bindgen::to_value(&CanvasContextOptions {
                desynchronized: false,
            })
            .unwrap(),
        )
        .unwrap()
        .unwrap()
        .dyn_into::<CanvasRenderingContext2d>()
        .unwrap();
    let upload_input = utils::get_element_by_id::<HtmlInputElement>(&document, "upload");
    let start_input = utils::get_element_by_id::<HtmlInputElement>(&document, "start");
    let change_speed_input =
        utils::get_element_by_id::<HtmlInputElement>(&document, "change-speed");
    let change_step_input = utils::get_element_by_id::<HtmlInputElement>(&document, "change-step");
    let image_dimensions = Default::default();
    let raf_handle = RefCell::new(None);
    let path_len = Cell::new(None);

    let global_state = Rc::new(GlobalState {
        ctx,
        upload_input,
        start_input,
        worker,
        image_dimensions,
        raf_handle,
        path_len,
        change_speed_input,
        change_step_input,
    });

    let step_input = utils::get_element_by_id::<HtmlInputElement>(&document, "step");
    let stop_input = utils::get_element_by_id::<HtmlInputElement>(&document, "stop");

    let local_state = LocalState {
        step_input,
        stop_input,
    };

    handlers::initialize_event_listeners(global_state, local_state);
}

#[derive(Serialize)]
struct CanvasContextOptions {
    desynchronized: bool,
}
