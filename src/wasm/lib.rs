use js_sys::Array;
use serde::{Deserialize, Serialize};
use std::{cell::OnceCell, rc::Rc};
use wasm_bindgen::prelude::*;
use web_sys::{
    CanvasRenderingContext2d, HtmlCanvasElement, HtmlInputElement, Worker, WorkerOptions,
    WorkerType,
};

mod gilbert;
mod handlers;
mod renderer;
mod utils;
mod worker;

#[derive(Debug)]
struct DOMState {
    canvas: HtmlCanvasElement,
    ctx: CanvasRenderingContext2d,
    upload_input: HtmlInputElement,
    start_input: HtmlInputElement,
    step_input: HtmlInputElement,
    stop_input: HtmlInputElement,
    change_speed_input: HtmlInputElement,
    change_step_input: HtmlInputElement,
}

thread_local! {
    // static DOM: OnceCell<Rc<DOMState>> = const { OnceCell::new() };
    static WORKER: OnceCell<Rc<Worker>> = const { OnceCell::new() };
}

#[wasm_bindgen(start)]
fn start() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();

    let document = web_sys::window().unwrap().document().unwrap();
    let canvas = Rc::new(
        document
            .query_selector("canvas")
            .unwrap()
            .unwrap()
            .dyn_into::<HtmlCanvasElement>()
            .unwrap(),
    );
    #[derive(Serialize)]
    struct CanvasContextOptions {
        desynchronized: bool,
    }
    let ctx = Rc::new(
        canvas
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
            .unwrap(),
    );
    let upload_input = Rc::new(
        document
            .get_element_by_id("upload")
            .unwrap()
            .dyn_into::<HtmlInputElement>()
            .unwrap(),
    );
    let start_input = Rc::new(
        document
            .get_element_by_id("start")
            .unwrap()
            .dyn_into::<HtmlInputElement>()
            .unwrap(),
    );
    let step_input = document
        .get_element_by_id("step")
        .unwrap()
        .dyn_into::<HtmlInputElement>()
        .unwrap();
    let stop_input = document
        .get_element_by_id("stop")
        .unwrap()
        .dyn_into::<HtmlInputElement>()
        .unwrap();
    let change_speed_input = document
        .get_element_by_id("change-speed")
        .unwrap()
        .dyn_into::<HtmlInputElement>()
        .unwrap();
    let change_step_input = document
        .get_element_by_id("change-step")
        .unwrap()
        .dyn_into::<HtmlInputElement>()
        .unwrap();
    // DOM.with(|dom| {
    //     dom.set(Rc::new(DOMState {
    //         canvas,
    //         ctx,
    //         upload_input,
    //         start_input,
    //         step_input,
    //         stop_input,
    //         change_speed_input,
    //         change_step_input,
    //     }))
    //     .unwrap();
    // });

    let worker_options = WorkerOptions::new();
    worker_options.set_type(WorkerType::Module);
    let worker = Rc::new(Worker::new_with_options("./worker.js", &worker_options).unwrap());
    WORKER.with(|worker_cell| {
        worker_cell.set(worker.clone()).unwrap();
    });

    let worker_message = Array::new();
    worker_message.push(&wasm_bindgen::module());
    worker_message.push(&wasm_bindgen::memory());
    worker.post_message(&worker_message).unwrap();

    {
        let upload_input_clone = upload_input.clone();
        let canvas_clone = canvas.clone();
        let ctx_clone = ctx.clone();
        let closure = Closure::<dyn FnMut()>::new(move || {
            let upload_input_clone = upload_input_clone.clone();
            let canvas_clone = canvas_clone.clone();
            let ctx_clone = ctx_clone.clone();
            wasm_bindgen_futures::spawn_local(async move {
                handlers::uploaded_image(upload_input_clone, canvas_clone, ctx_clone).await;
            });
        });
        upload_input.set_onchange(Some(closure.as_ref().unchecked_ref()));
        closure.forget();
    }
}

#[wasm_bindgen(js_name = handleMessage)]
pub fn handle_message(message: JsValue) {
    let received_worker_message: ReceivedWorkerMessage =
        serde_wasm_bindgen::from_value(message).unwrap();
    received_worker_message.process();
}

#[derive(Deserialize)]
#[serde(tag = "action", content = "payload")]
enum ReceivedWorkerMessage {
    // #[serde(rename = "loadImage")]
    // LoadImage,
    #[serde(rename = "step")]
    Step,
    #[serde(rename = "start")]
    Start,
    #[serde(rename = "stop")]
    Stop,
    #[serde(rename = "changeSpeed")]
    ChangeSpeed(ChangeSpeedMessage),
    #[serde(rename = "changeStep")]
    ChangeStep(ChangeStepMessage),
}

#[derive(Deserialize)]
struct CanvasInitMessage {
    #[serde(with = "serde_wasm_bindgen::preserve")]
    canvas: HtmlCanvasElement,
}

#[derive(Deserialize)]
struct ChangeSpeedMessage {
    new_speed_percentage: usize,
}

#[derive(Deserialize)]
struct ChangeStepMessage {
    new_step_percentage: usize,
}

impl ReceivedWorkerMessage {
    pub fn process(self) {
        match self {
            // Self::LoadImage => {
            //     renderer::load_image();
            // }
            Self::Stop => {
                renderer::stop();
            }
            Self::ChangeSpeed(change_speed_message) => {
                renderer::change_speed(change_speed_message);
            }
            Self::ChangeStep(change_step_message) => {
                renderer::change_step(change_step_message);
            }

            Self::Step => {
                worker::step();
            }
            Self::Start => {
                worker::start();
            }
        }
    }
}
