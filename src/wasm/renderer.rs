use crate::{gilbert, worker, CanvasInitMessage, ChangeSpeedMessage, ChangeStepMessage};
use js_sys::{Uint8ClampedArray, WebAssembly};
use serde::Serialize;
use std::{cell::OnceCell, ptr, rc::Rc, num::NonZero};
use wasm_bindgen::prelude::*;
use web_sys::CanvasRenderingContext2d;

thread_local! {
    static CTX: OnceCell<Rc<CanvasRenderingContext2d>> = const { OnceCell::new() };
}
pub static mut WIDTH: usize = 0;
pub static mut CURVE: Vec<usize> = Vec::new();
pub static mut HEIGHT: usize = 0;
pub static mut PIXEL_DATA: Option<Vec<u8>> = None;

#[derive(Serialize)]
pub struct CanvasContextOptions {
    pub desynchronized: bool,
}

pub fn load_image() {
    let width = 512;
    let height = 512;
    let ctx = CTX.with(|ctx| ctx.get().unwrap().clone());
    let pixel_data = ctx
        .get_image_data(0.0, 0.0, width as f64, height as f64)
        .unwrap()
        .data()
        .0;
    unsafe {
        CURVE = (0..(width * height))
            .map(|idx| {
                let p = gilbert::gilbert_d2xy(idx as i32, width as i32, height as i32);
                ((p.y as usize) * width + (p.x as usize)) * 4
            })
            .collect();
        PIXEL_DATA = Some(pixel_data);
        WIDTH = width;
        HEIGHT = height;
    }
}

pub fn canvas_init(canvas_init_message: CanvasInitMessage) {
    CTX.with(|ctx| {
        ctx.get_or_init(|| {
            Rc::new(
                canvas_init_message
                    .canvas
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
            )
        });
    });
}

#[wasm_bindgen]
extern "C" {
    #[derive(Debug)]
    type ImageData;

    #[wasm_bindgen(constructor, catch)]
    fn new(data: &Uint8ClampedArray, width: u32, height: u32) -> Result<ImageData, JsValue>;
}

#[wasm_bindgen(js_name = renderPixelData)]
pub fn render_pixel_data() {
    let base = unsafe { PIXEL_DATA.as_ref().unwrap().as_ptr() } as u32;
    let len = unsafe { PIXEL_DATA.as_ref().unwrap().len() } as u32;
    let sliced_pixel_data = Uint8ClampedArray::new(
        &wasm_bindgen::memory()
            .unchecked_into::<WebAssembly::Memory>()
            .buffer(),
    )
    .slice(base, base + len);

    let image_data = &ImageData::new(
        &sliced_pixel_data,
        unsafe { WIDTH } as u32,
        unsafe { HEIGHT } as u32,
    )
    .unwrap()
    .dyn_into::<web_sys::ImageData>()
    .unwrap();

    CTX.with(|ctx| {
        ctx.get()
            .unwrap()
            .put_image_data(image_data, 0.0, 0.0)
            .unwrap();
    });
}

pub fn stop() {
    unsafe {
        worker::STOP_WORKER_LOOP = true;
        while ptr::read_volatile(ptr::addr_of!(worker::STOP_WORKER_LOOP)) {}
    }
    render_pixel_data();
}

const ALL_SLEEPS_PER_LOOP: [usize; 10] =
    [200_000, 175_000, 50_000, 10_000, 2500, 500, 40, 20, 10, 0];

pub fn change_speed(change_speed_message: ChangeSpeedMessage) {
    let lerped: u64 = crate::utils::lerp(
        ALL_SLEEPS_PER_LOOP,
        change_speed_message.new_speed_percentage,
    );
    unsafe {
        worker::SLEEP_PER_LOOP = lerped;
    }
}

const ALL_STEPS_PER_LOOP: [isize; 13] = [
    -1000, -700, -300, -100, -50, -5, 0, 5, 50, 100, 300, 700, 1000,
];
pub fn change_step(change_step_message: ChangeStepMessage) {
    let lerped: isize =
        crate::utils::lerp(ALL_STEPS_PER_LOOP, change_step_message.new_step_percentage);
    unsafe {
        worker::STEPS_PER_LOOP = NonZero::new(lerped).unwrap_or(NonZero::new(1).unwrap());
    }
}
