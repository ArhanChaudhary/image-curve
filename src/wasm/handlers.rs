use crate::{renderer, utils, worker, GlobalState, LocalState};
use js_sys::{Function, Promise};
use serde::{Deserialize, Serialize};
use std::rc::Rc;
use wasm_bindgen::JsCast;
use wasm_bindgen::{prelude::Closure, JsValue};
use web_sys::{HtmlImageElement, HtmlInputElement, MessageEvent, PointerEvent};

pub fn initialize_event_listeners(global_state: Rc<GlobalState>, local_state: LocalState) {
    {
        let global_state_clone = global_state.clone();
        let onchange_closure = Closure::<dyn Fn()>::new(move || {
            let global_state_clone = global_state_clone.clone();
            wasm_bindgen_futures::spawn_local(async move {
                uploaded_image(global_state_clone.clone()).await;
            });
        });
        global_state
            .upload_input
            .add_event_listener_with_callback("change", onchange_closure.as_ref().unchecked_ref())
            .unwrap();
        onchange_closure.forget();
    }

    {
        let global_state_clone = global_state.clone();
        let onclick_closure = Closure::<dyn Fn()>::new(move || {
            clicked_start(global_state_clone.clone());
        });
        global_state
            .start_input
            .add_event_listener_with_callback("click", onclick_closure.as_ref().unchecked_ref())
            .unwrap();
        onclick_closure.forget();
    }

    {
        let global_state_clone = global_state.clone();
        let onclick_closure = Closure::<dyn Fn()>::new(move || {
            clicked_stop(global_state_clone.clone());
        });
        local_state
            .stop_input
            .add_event_listener_with_callback("click", onclick_closure.as_ref().unchecked_ref())
            .unwrap();
        onclick_closure.forget();
    }

    {
        let global_state_clone = global_state.clone();
        let onclick_closure = Closure::<dyn Fn()>::new(move || {
            let global_state_clone = global_state_clone.clone();
            wasm_bindgen_futures::spawn_local(async move {
                clicked_step(global_state_clone.clone()).await;
            });
        });
        local_state
            .step_input
            .add_event_listener_with_callback("click", onclick_closure.as_ref().unchecked_ref())
            .unwrap();
        onclick_closure.forget();
    }

    {
        let oninput_closure = Closure::<dyn Fn(_)>::new(move |e: PointerEvent| {
            inputted_speed(e);
        });
        local_state
            .change_speed_input
            .add_event_listener_with_callback("input", oninput_closure.as_ref().unchecked_ref())
            .unwrap();
        oninput_closure.forget();
    }

    {
        let global_state_clone = global_state.clone();
        let oninput_closure = Closure::<dyn Fn(_)>::new(move |e: PointerEvent| {
            inputted_step(e, global_state_clone.clone());
        });
        local_state
            .change_step_input
            .add_event_listener_with_callback("input", oninput_closure.as_ref().unchecked_ref())
            .unwrap();
        oninput_closure.forget();
    }
}

pub async fn uploaded_image(global_state: Rc<GlobalState>) {
    let src =
        crate::utils::to_base64(global_state.upload_input.files().unwrap().get(0).unwrap()).await;
    let img = HtmlImageElement::new().unwrap();
    img.set_src(&src);
    let promise = Promise::new(&mut |resolve: Function, _reject: Function| {
        img.set_onload(Some(&resolve));
    });
    wasm_bindgen_futures::JsFuture::from(promise).await.unwrap();
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
    id: i32,
    closure: Closure<dyn FnMut()>,
}

impl Drop for RequestAnimationFrameHandle {
    fn drop(&mut self) {
        web_sys::window()
            .unwrap()
            .cancel_animation_frame(self.id)
            .unwrap();
    }
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
        .value_as_number() as u32;
    renderer::change_speed(new_speed_percentage);
}

pub fn inputted_step(e: PointerEvent, global_state: Rc<GlobalState>) {
    let new_step_percentage = e
        .target()
        .unwrap()
        .unchecked_into::<HtmlInputElement>()
        .value_as_number() as u32;
    renderer::change_step(new_step_percentage, global_state);
}
