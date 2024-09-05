use crate::{renderer, utils, worker, GlobalState};
use js_sys::{Function, Promise};
use serde::{Deserialize, Serialize};
use std::rc::Rc;
use wasm_bindgen::JsCast;
use wasm_bindgen::{prelude::Closure, JsValue};
use web_sys::{HtmlImageElement, HtmlInputElement, MessageEvent, PointerEvent};

pub async fn uploaded_image(global_state: Rc<GlobalState>) {
    let src =
        crate::utils::to_base64(global_state.upload_input.files().unwrap().get(0).unwrap()).await;
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
    let canvas = global_state.ctx.canvas().unwrap();
    canvas.set_width(width);
    canvas.set_height(height);
    global_state
        .ctx
        .draw_image_with_html_image_element_and_dw_and_dh(
            &img,
            0.0,
            0.0,
            width as f64,
            height as f64,
        )
        .unwrap();
    renderer::load_image(global_state);
    // change speed / step here TODO:
}

pub struct RequestAnimationFrameHandle {
    pub id: i32,
    pub closure: Closure<dyn FnMut()>,
}

pub fn clicked_start(global_state: Rc<GlobalState>) {
    if global_state.raf_handle.borrow().is_some() {
        return;
    }
    global_state
        .worker
        .post_message(&serde_wasm_bindgen::to_value(&worker::WorkerMessage::Start).unwrap())
        .unwrap();

    let global_state_clone = global_state.clone();
    let render_pixel_data_loop = Closure::<dyn FnMut()>::new(move || {
        renderer::render_pixel_data(global_state_clone.clone());
        let id = utils::request_animation_frame(
            &global_state_clone
                .raf_handle
                .borrow()
                .as_ref()
                .unwrap()
                .closure,
        );
        global_state_clone
            .raf_handle
            .borrow_mut()
            .as_mut()
            .unwrap()
            .id = id;
    });
    *global_state.raf_handle.borrow_mut() = Some(RequestAnimationFrameHandle {
        id: utils::request_animation_frame(&render_pixel_data_loop),
        closure: render_pixel_data_loop,
    });
}

pub fn clicked_stop(global_state: Rc<GlobalState>) {
    renderer::stop(global_state.clone());
    let taken = global_state.raf_handle.borrow_mut().take().unwrap();
    utils::cancel_animation_frame(taken.id);
}

#[derive(Serialize, Deserialize, PartialEq)]
#[serde(tag = "action", content = "payload")]
pub enum MainMessage {
    Stepped,
}

pub async fn clicked_step(global_state: Rc<GlobalState>) {
    if global_state.raf_handle.borrow().is_some() {
        return;
    }
    global_state
        .worker
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
            global_state
                .worker
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

    renderer::render_pixel_data(global_state);
}

pub fn inputted_speed(e: PointerEvent) {
    let new_speed_percentage = e
        .target()
        .unwrap()
        .unchecked_into::<HtmlInputElement>()
        .value_as_number() as usize;
    renderer::change_speed(new_speed_percentage)
}

pub fn inputted_step(e: PointerEvent, global_state: Rc<GlobalState>) {
    let change_step_message = e
        .target()
        .unwrap()
        .unchecked_into::<HtmlInputElement>()
        .value_as_number() as usize;
    renderer::change_step(change_step_message, global_state);
}
