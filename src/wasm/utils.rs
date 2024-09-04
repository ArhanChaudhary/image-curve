use js_sys::{Function, JsString, Promise};
use num::{Integer, Num, NumCast};
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use web_sys::{Document, File, FileReader};

pub fn lerp<T: Integer + NumCast + Copy, const N: usize, R: Num + NumCast>(
    values: [T; N],
    percentage: usize,
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

pub fn cancel_animation_frame(id: i32) {
    web_sys::window()
        .unwrap()
        .cancel_animation_frame(id)
        .unwrap();
}

pub async fn to_base64(file: File) -> String {
    let reader = Rc::new(FileReader::new().unwrap());
    reader.read_as_data_url(&file).unwrap();

    let promise = Promise::new(&mut |resolve: Function, reject: Function| {
        let reader_clone = reader.clone();
        let onload = Closure::once_into_js(move || {
            let Ok(result) = reader_clone.result() else {
                reject.call0(&JsValue::NULL).unwrap();
                return;
            };
            let Ok(result) = result.dyn_into::<JsString>() else {
                reject.call0(&JsValue::NULL).unwrap();
                return;
            };
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
