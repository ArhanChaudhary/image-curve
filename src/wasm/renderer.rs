// use std::sync::OnceLock;

use crate::gilbert::gilbert_d2xy;
use js_sys::{Uint8ClampedArray, WebAssembly};
use serde::Serialize;
// use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use web_sys::DedicatedWorkerGlobalScope;
// use web_sys::OffscreenCanvas;

// static CTX: OnceLock<OffscreenCanvasRenderingContext2d> = OnceLock::new();
static mut CURVE: Vec<usize> = Vec::new();
static mut PIXEL_DATA: Vec<u8> = Vec::new();
static mut WIDTH: usize = 0;
static mut HEIGHT: usize = 0;

// #[derive(Deserialize)]
// pub struct DeserializeableOffscreenCanvas(
//     #[serde(with = "serde_wasm_bindgen::preserve", rename = "offscreenCanvas")] OffscreenCanvas,
// );

// #[derive(Serialize)]
// pub struct CanvasContextOptions {
//     pub desynchronized: bool,
// }

#[wasm_bindgen]
extern "C" {
    type ImageData;
    pub type OffscreenCanvasRenderingContext2d;

    #[wasm_bindgen(constructor)]
    fn new_with_uint8_clamped_array_and_width_and_height(
        data: Uint8ClampedArray,
        width: usize,
        height: usize,
    ) -> ImageData;

    #[wasm_bindgen(method, js_name = putImageData)]
    fn put_image_data(
        this: &OffscreenCanvasRenderingContext2d,
        image_data: ImageData,
        dx: usize,
        dy: usize,
    );

    #[wasm_bindgen(method, js_name = clearRect)]
    fn clear_rect(
        this: &OffscreenCanvasRenderingContext2d,
        x: usize,
        y: usize,
        width: usize,
        height: usize,
    );
}

unsafe impl Sync for OffscreenCanvasRenderingContext2d {}
unsafe impl Send for OffscreenCanvasRenderingContext2d {}

// pub fn init(init_message: InitMessage) {
//     CTX.get_or_init(|| {
//         init_message
//             .offscreen_canvas
//             .0
//             .get_context_with_context_options(
//                 "2d",
//                 &serde_wasm_bindgen::to_value(&CanvasContextOptions {
//                     desynchronized: false,
//                 })
//                 .unwrap(),
//             )
//             .unwrap()
//             .unwrap()
//             .unchecked_into::<OffscreenCanvasRenderingContext2d>()
//     });
// }

pub fn canvas_init(width: usize, height: usize, pixel_data: Vec<u8>) {
    unsafe {
        CURVE = (0..(width * height))
            .map(|idx| {
                let p = gilbert_d2xy(idx as i32, width as i32, height as i32);
                ((p.y as usize) * width + (p.x as usize)) * 4
            })
            .collect();
        PIXEL_DATA = pixel_data;
        WIDTH = width;
        HEIGHT = height;
    }
    render_pixel_data();
}

pub fn step() {
    let width = unsafe { WIDTH };
    let height = unsafe { HEIGHT };
    let curve = unsafe { CURVE.as_mut_ptr() };
    let pixel_data = unsafe { PIXEL_DATA.as_mut_ptr() };

    let n = width * height;
    let step = 1;
    unsafe {
        for curve_index in 0..(n - step) {
            let pixel_index = *curve.add(curve_index);
            let prev_pixel_index = *curve.add((curve_index + n - step) % n);

            let temp_r = *pixel_data.add(pixel_index);
            *pixel_data.add(prev_pixel_index) = temp_r;
            *pixel_data.add(pixel_index) = *pixel_data.add(prev_pixel_index);

            let temp_g = *pixel_data.add(pixel_index + 1);
            *pixel_data.add(prev_pixel_index + 1) = temp_g;
            *pixel_data.add(pixel_index + 1) = *pixel_data.add(prev_pixel_index + 1);

            let temp_b = *pixel_data.add(pixel_index + 2);
            *pixel_data.add(prev_pixel_index + 2) = temp_b;
            *pixel_data.add(pixel_index + 2) = *pixel_data.add(prev_pixel_index + 2);
        }
    }
    render_pixel_data();
}

fn render_pixel_data() {
    let sliced_pixel_data = Uint8ClampedArray::new(
        &wasm_bindgen::memory()
            .unchecked_into::<WebAssembly::Memory>()
            .buffer(),
    )
    .slice(unsafe { PIXEL_DATA.as_ptr() as u32 }, unsafe {
        (PIXEL_DATA.as_ptr() as u32) + (PIXEL_DATA.len() as u32)
    });

    // CTX.get().unwrap().put_image_data(
    //     ImageData::new_with_uint8_clamped_array_and_width_and_height(
    //         sliced_pixel_data,
    //         unsafe { WIDTH },
    //         unsafe { HEIGHT },
    //     ),
    //     0,
    //     0,
    // );
    let _ = js_sys::global()
        .unchecked_into::<DedicatedWorkerGlobalScope>()
        .post_message(
            &serde_wasm_bindgen::to_value(&SendImageMessage {
                image_buffer: sliced_pixel_data,
            })
            .unwrap(),
        );
}

#[derive(Serialize)]
struct SendImageMessage {
    #[serde(with = "serde_wasm_bindgen::preserve", rename = "imageBuffer")]
    image_buffer: Uint8ClampedArray,
}
