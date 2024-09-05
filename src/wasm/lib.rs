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
    CanvasRenderingContext2d, HtmlCanvasElement, HtmlInputElement, PointerEvent, Worker,
    WorkerOptions, WorkerType,
};

mod gilbert;
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
    let step_input = utils::get_element_by_id::<HtmlInputElement>(&document, "step");
    let stop_input = utils::get_element_by_id::<HtmlInputElement>(&document, "stop");
    let change_speed_input =
        utils::get_element_by_id::<HtmlInputElement>(&document, "change-speed");
    let change_step_input = utils::get_element_by_id::<HtmlInputElement>(&document, "change-step");

    let worker_options = WorkerOptions::new();
    worker_options.set_type(WorkerType::Module);
    let worker = Worker::new_with_options("./worker.js", &worker_options).unwrap();

    let worker_message = Array::new();
    worker_message.push(&wasm_bindgen::module());
    worker_message.push(&wasm_bindgen::memory());
    worker.post_message(&worker_message).unwrap();

    let image_dimensions = OnceCell::new();
    let raf_handle = RefCell::new(None);

    let global_state = Rc::new(GlobalState {
        ctx,
        upload_input,
        start_input,
        worker,
        image_dimensions,
        raf_handle,
    });
    {
        let global_state_clone = global_state.clone();
        let onchange_closure = Closure::<dyn Fn()>::new(move || {
            let global_state_clone = global_state_clone.clone();
            wasm_bindgen_futures::spawn_local(async move {
                handlers::uploaded_image(global_state_clone.clone()).await;
            });
        });
        global_state
            .upload_input
            .set_onchange(Some(onchange_closure.as_ref().unchecked_ref()));
        onchange_closure.forget();
    }

    {
        let global_state_clone = global_state.clone();
        let onclick_closure = Closure::<dyn Fn()>::new(move || {
            handlers::clicked_start(global_state_clone.clone());
        });
        global_state
            .start_input
            .set_onclick(Some(onclick_closure.as_ref().unchecked_ref()));
        onclick_closure.forget();
    }

    {
        let global_state_clone = global_state.clone();
        let onclick_closure = Closure::<dyn Fn()>::new(move || {
            handlers::clicked_stop(global_state_clone.clone());
        });
        stop_input.set_onclick(Some(onclick_closure.as_ref().unchecked_ref()));
        onclick_closure.forget();
    }

    {
        let global_state_clone = global_state.clone();
        let onclick_closure = Closure::<dyn Fn()>::new(move || {
            let global_state_clone = global_state_clone.clone();
            wasm_bindgen_futures::spawn_local(async move {
                handlers::clicked_step(global_state_clone.clone()).await;
            });
        });
        step_input.set_onclick(Some(onclick_closure.as_ref().unchecked_ref()));
        onclick_closure.forget();
    }

    {
        let oninput_closure = Closure::<dyn Fn(_)>::new(move |e: PointerEvent| {
            handlers::inputted_speed(e);
        });
        change_speed_input.set_oninput(Some(oninput_closure.as_ref().unchecked_ref()));
        oninput_closure.forget();
    }

    {
        let global_state_clone = global_state.clone();
        let oninput_closure = Closure::<dyn Fn(_)>::new(move |e: PointerEvent| {
            handlers::inputted_step(e, global_state_clone.clone());
        });
        change_step_input.set_oninput(Some(oninput_closure.as_ref().unchecked_ref()));
        oninput_closure.forget();
    }
}

#[derive(Serialize)]
struct CanvasContextOptions {
    desynchronized: bool,
}
