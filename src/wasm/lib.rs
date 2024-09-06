use handlers::RequestAnimationFrameHandle;
use js_sys::Array;
use renderer::ImageDimensions;
use serde::Serialize;
use std::{
    cell::{OnceCell, RefCell},
    rc::Rc,
};
use wasm_bindgen::prelude::*;
use web_sys::{
    CanvasRenderingContext2d, HtmlCanvasElement, HtmlInputElement, Worker, WorkerOptions,
    WorkerType,
};

mod paths;
mod handlers;
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
    image_dimensions: OnceCell<ImageDimensions>,
    raf_handle: RefCell<Option<RequestAnimationFrameHandle>>,
}

struct LocalState {
    step_input: HtmlInputElement,
    stop_input: HtmlInputElement,
    change_speed_input: HtmlInputElement,
    change_step_input: HtmlInputElement,
}

#[wasm_bindgen(js_name = runMain)]
pub fn run_main() {
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
    let image_dimensions = OnceCell::new();
    let raf_handle = RefCell::new(None);

    let worker_options = WorkerOptions::new();
    worker_options.set_type(WorkerType::Module);
    let worker = Worker::new_with_options("./worker.js", &worker_options).unwrap();
    let worker_message = Array::new();
    worker_message.push(&wasm_bindgen::module());
    worker_message.push(&wasm_bindgen::memory());
    worker.post_message(&worker_message).unwrap();

    let global_state = Rc::new(GlobalState {
        ctx,
        upload_input,
        start_input,
        worker,
        image_dimensions,
        raf_handle,
    });

    let step_input = utils::get_element_by_id::<HtmlInputElement>(&document, "step");
    let stop_input = utils::get_element_by_id::<HtmlInputElement>(&document, "stop");
    let change_speed_input =
        utils::get_element_by_id::<HtmlInputElement>(&document, "change-speed");
    let change_step_input = utils::get_element_by_id::<HtmlInputElement>(&document, "change-step");

    let local_state = LocalState {
        step_input,
        stop_input,
        change_speed_input,
        change_step_input,
    };

    handlers::initialize_handlers(global_state, local_state);
}

#[derive(Serialize)]
struct CanvasContextOptions {
    desynchronized: bool,
}
