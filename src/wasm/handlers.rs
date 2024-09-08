use crate::{renderer, utils, worker, GlobalState, LocalState};
use js_sys::{Function, Promise};
use serde::{Deserialize, Serialize};
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use web_sys::HtmlImageElement;

pub fn initialize_event_listeners(global_state: Rc<GlobalState>, local_state: LocalState) {
    {
        let global_state_clone = global_state.clone();
        let onchange_closure = Closure::<dyn Fn()>::new(move || {
            let global_state_clone = global_state_clone.clone();
            wasm_bindgen_futures::spawn_local(async move {
                uploaded_image(&global_state_clone).await;
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
            let global_state_clone = global_state_clone.clone();
            wasm_bindgen_futures::spawn_local(async move {
                clicked_stop(&global_state_clone).await;
            });
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
                clicked_step(&global_state_clone).await;
            });
        });
        local_state
            .step_input
            .add_event_listener_with_callback("click", onclick_closure.as_ref().unchecked_ref())
            .unwrap();
        onclick_closure.forget();
    }

    {
        let global_state_clone = global_state.clone();
        let oninput_closure = Closure::<dyn Fn()>::new(move || {
            inputted_speed(&global_state_clone);
        });
        global_state
            .change_speed_input
            .add_event_listener_with_callback("input", oninput_closure.as_ref().unchecked_ref())
            .unwrap();
        oninput_closure.forget();
    }

    {
        let global_state_clone = global_state.clone();
        let oninput_closure = Closure::<dyn Fn()>::new(move || {
            inputted_step(&global_state_clone);
        });
        global_state
            .change_step_input
            .add_event_listener_with_callback("input", oninput_closure.as_ref().unchecked_ref())
            .unwrap();
        oninput_closure.forget();
    }
}

pub async fn uploaded_image(global_state: &GlobalState) {
    let Some(file) = global_state.upload_input.files().unwrap().get(0) else {
        return;
    };
    let src = crate::utils::to_base64(file).await;
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
    renderer::load_image(global_state).await;
    inputted_speed(global_state);
    inputted_step(global_state);
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
        renderer::render_pixel_data(&global_state_clone);
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

pub async fn clicked_stop(global_state: &GlobalState) {
    if global_state.raf_handle.borrow().is_none() {
        return;
    }
    renderer::stop(global_state).await;
    global_state.raf_handle.borrow_mut().take().unwrap();
}

#[derive(Serialize, Deserialize, PartialEq, Copy, Clone, Debug)]
#[serde(tag = "action", content = "payload")]
pub enum MainMessage {
    Stepped,
    Stopped,
    LoadedPath { path_len: u32 },
}

pub async fn clicked_step(global_state: &GlobalState) {
    if global_state.raf_handle.borrow().is_some() {
        return;
    }
    let received_worker_message =
        utils::worker_operation(&global_state.worker, worker::WorkerMessage::Step).await;
    if received_worker_message != MainMessage::Stepped {
        panic!(
            "Expected MainMessage::Stepped, got {:?}",
            received_worker_message
        );
    }
    renderer::render_pixel_data(global_state);
}

pub fn inputted_speed(global_state: &GlobalState) {
    let new_speed_percentage = global_state.change_speed_input.value_as_number() as u32;
    renderer::change_speed(new_speed_percentage);
}

pub fn inputted_step(global_state: &GlobalState) {
    if global_state.path_len.get().is_none() {
        return;
    }
    let new_step_percentage = global_state.change_step_input.value_as_number() as u32;
    renderer::change_step(new_step_percentage, global_state);
}
