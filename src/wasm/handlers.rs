use crate::renderer::ImageDimensions;
use crate::{renderer, utils, worker};
use js_sys::{Function, Promise};
use serde::{Deserialize, Serialize};
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use wasm_bindgen::JsCast;
use wasm_bindgen::{prelude::Closure, JsValue};
use web_sys::{
    CanvasRenderingContext2d, HtmlImageElement, HtmlInputElement, MessageEvent, PointerEvent,
    Worker,
};

pub async fn uploaded_image(
    upload_input: Rc<HtmlInputElement>,
    ctx: Rc<CanvasRenderingContext2d>,
    image_dimensions: Rc<Cell<Option<ImageDimensions>>>,
) {
    let src = crate::utils::to_base64(upload_input.files().unwrap().get(0).unwrap()).await;
    let img = HtmlImageElement::new().unwrap();
    img.set_src(&src);
    wasm_bindgen_futures::JsFuture::from(Promise::new(
        &mut |resolve: Function, _reject: Function| {
            img.set_onload(Some(&resolve));
        },
    ))
    .await
    .unwrap();
    let width = img.width();
    let height = img.height();
    let canvas = ctx.canvas().unwrap();
    canvas.set_width(width);
    canvas.set_height(height);
    ctx.draw_image_with_html_image_element_and_dw_and_dh(
        &img,
        0.0,
        0.0,
        width as f64,
        height as f64,
    )
    .unwrap();
    renderer::load_image(ctx, image_dimensions);
    // change speed / step here TODO:
}

pub struct RequestAnimationFrameHandle {
    pub id: i32,
    pub closure: Closure<dyn FnMut()>,
}

pub fn clicked_start(
    ctx: Rc<CanvasRenderingContext2d>,
    worker: Rc<Worker>,
    raf_handle: Rc<RefCell<Option<RequestAnimationFrameHandle>>>,
    image_dimensions: Rc<Cell<Option<ImageDimensions>>>,
) {
    if raf_handle.borrow().is_some() {
        return;
    }
    worker
        .post_message(&serde_wasm_bindgen::to_value(&worker::WorkerMessage::Start).unwrap())
        .unwrap();

    let raf_handle_clone = raf_handle.clone();
    let render_pixel_data_loop = Closure::<dyn FnMut()>::new(move || {
        renderer::render_pixel_data(ctx.clone(), image_dimensions.clone());
        let id =
            utils::request_animation_frame(&raf_handle_clone.borrow().as_ref().unwrap().closure);
        raf_handle_clone.borrow_mut().as_mut().unwrap().id = id;
    });
    *raf_handle.borrow_mut() = Some(RequestAnimationFrameHandle {
        id: utils::request_animation_frame(&render_pixel_data_loop),
        closure: render_pixel_data_loop,
    });
}

pub fn clicked_stop(
    ctx: Rc<CanvasRenderingContext2d>,
    raf_handle: Rc<RefCell<Option<RequestAnimationFrameHandle>>>,
    image_dimensions: Rc<Cell<Option<ImageDimensions>>>,
) {
    renderer::stop(ctx, image_dimensions);
    let taken = raf_handle.borrow_mut().take().unwrap();
    utils::cancel_animation_frame(taken.id);
}

#[derive(Serialize, Deserialize, PartialEq)]
#[serde(tag = "action", content = "payload")]
pub enum MainMessage {
    Stepped,
}

pub async fn clicked_step(
    ctx: Rc<CanvasRenderingContext2d>,
    worker: Rc<Worker>,
    raf_handle: Rc<RefCell<Option<RequestAnimationFrameHandle>>>,
    image_dimensions: Rc<Cell<Option<ImageDimensions>>>,
) {
    if raf_handle.borrow().is_some() {
        return;
    }
    worker
        .post_message(&serde_wasm_bindgen::to_value(&worker::WorkerMessage::Step).unwrap())
        .unwrap();

    wasm_bindgen_futures::JsFuture::from(Promise::new(
        &mut |resolve: Function, reject: Function| {
            let closure = Closure::once_into_js(move |e: MessageEvent| {
                let message = e.data();
                let Ok(received_worker_message) =
                    serde_wasm_bindgen::from_value::<MainMessage>(message)
                else {
                    reject.call0(&JsValue::NULL).unwrap();
                    return;
                };

                if received_worker_message == MainMessage::Stepped {
                    resolve.call0(&JsValue::NULL).unwrap();
                } else {
                    reject.call0(&JsValue::NULL).unwrap();
                }
            });

            let event_listener_options = web_sys::AddEventListenerOptions::new();
            event_listener_options.set_once(true);
            worker
                .add_event_listener_with_callback_and_add_event_listener_options(
                    "message",
                    closure.unchecked_ref(),
                    &event_listener_options,
                )
                .unwrap();
        },
    ))
    .await
    .unwrap();

    renderer::render_pixel_data(ctx, image_dimensions);
}

pub fn inputted_speed(e: PointerEvent) {
    let new_speed_percentage = e
        .target()
        .unwrap()
        .unchecked_into::<HtmlInputElement>()
        .value_as_number() as usize;
    renderer::change_speed(new_speed_percentage)
}

pub fn inputted_step(e: PointerEvent, image_dimensions: Rc<Cell<Option<ImageDimensions>>>) {
    let change_step_message = e
        .target()
        .unwrap()
        .unchecked_into::<HtmlInputElement>()
        .value_as_number() as usize;
    renderer::change_step(change_step_message, image_dimensions)
}
