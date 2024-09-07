use crate::handlers::MainMessage;
use js_sys::{Function, JsString, Promise};
use num::{Integer, Num, NumCast};
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use web_sys::{Document, File, FileReader, MessageEvent, Worker};

pub fn lerp<T: Integer + NumCast + Copy, const N: usize, R: Num + NumCast>(
    values: [T; N],
    percentage: u32,
) -> R {
    let percentage_jump = 100.0 / (N as f64 - 1.0);
    let floored_index = (percentage as f64 / percentage_jump) as usize;
    if floored_index == N - 1 {
        return num::cast(values[N - 1]).unwrap();
    }
    let floored_val = values[floored_index].to_f64().unwrap();
    let ceiled_val = values[floored_index + 1].to_f64().unwrap();
    let lerp_percentage =
        (percentage as f64 - percentage_jump * floored_index as f64) / percentage_jump;
    let lerp = ((ceiled_val - floored_val) * lerp_percentage + floored_val).round();
    num::cast(lerp).unwrap()
}

pub fn get_element_by_id<T: JsCast>(document: &Document, id: &str) -> T {
    document.get_element_by_id(id).unwrap().dyn_into().unwrap()
}

pub fn request_animation_frame(f: &Closure<dyn FnMut()>) -> i32 {
    web_sys::window()
        .unwrap()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .unwrap()
}

pub async fn to_base64(file: File) -> String {
    let reader = Rc::new(FileReader::new().unwrap());
    reader.read_as_data_url(&file).unwrap();

    let promise = Promise::new(&mut |resolve: Function, _reject: Function| {
        let reader_clone = reader.clone();
        let onload = Closure::once_into_js(move || {
            let result = reader_clone
                .result()
                .unwrap_throw()
                .dyn_into::<JsString>()
                .unwrap_throw();
            resolve.call1(&JsValue::NULL, &result).unwrap();
        });
        reader.set_onload(Some(onload.unchecked_ref()));
    });

    wasm_bindgen_futures::JsFuture::from(promise)
        .await
        .unwrap()
        .dyn_into::<JsString>()
        .unwrap()
        .into()
}

pub async fn wait_for_worker_message(worker: &Worker, expected_worker_message: MainMessage) {
    let promise = Promise::new(&mut |resolve: Function, reject: Function| {
        let closure = Closure::once_into_js(move |e: MessageEvent| {
            let message = e.data();
            let received_worker_message = serde_wasm_bindgen::from_value::<MainMessage>(message).unwrap_throw();
            if received_worker_message == expected_worker_message {
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
    });
    wasm_bindgen_futures::JsFuture::from(promise).await.unwrap();
}
