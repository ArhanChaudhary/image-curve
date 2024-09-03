use crate::{gilbert, worker, ChangeSpeedMessage, ChangeStepMessage};
use js_sys::{Uint8ClampedArray, WebAssembly};
use std::{cell::OnceCell, ptr, rc::Rc};
use wasm_bindgen::prelude::*;
use web_sys::CanvasRenderingContext2d;

thread_local! {
    static CTX: OnceCell<Rc<CanvasRenderingContext2d>> = const { OnceCell::new() };
}
pub static mut WIDTH: Option<usize> = None;
pub static mut HEIGHT: Option<usize> = None;
pub static mut CURVE: Option<Vec<usize>> = None;
pub static mut PIXEL_DATA: Option<Vec<u8>> = None;

pub fn load_image(ctx: Rc<CanvasRenderingContext2d>) {
    // let ctx = CTX.with(|ctx| ctx.get().unwrap().clone());
    let width = ctx.canvas().unwrap().width() as usize;
    let height = ctx.canvas().unwrap().height() as usize;
    let pixel_data = ctx
        .get_image_data(0.0, 0.0, width as f64, height as f64)
        .unwrap()
        .data()
        .0;
    let curve = (0..(width * height))
        .map(|idx| {
            let p = gilbert::gilbert_d2xy(idx as i32, width as i32, height as i32);
            ((p.y as usize) * width + (p.x as usize)) * 4
        })
        .collect();
    unsafe {
        WIDTH = Some(width);
        HEIGHT = Some(height);
        CURVE = Some(curve);
        PIXEL_DATA = Some(pixel_data);
    }
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
        unsafe { WIDTH.unwrap() } as u32,
        unsafe { HEIGHT.unwrap() } as u32,
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

pub fn change_step(change_step_message: ChangeStepMessage) {
    unsafe {
        if WIDTH.is_none() || HEIGHT.is_none() {
            return;
        }
    }
    let mut scaled_step_percentage = (change_step_message.new_step_percentage as isize - 50) * 2;
    if scaled_step_percentage == 0 {
        scaled_step_percentage = 1;
    }
    let n = unsafe { WIDTH.unwrap() * HEIGHT.unwrap() };
    let n_proportion = 2_usize
        .pow(n.ilog2() - scaled_step_percentage.unsigned_abs() as u32 * (n.ilog2() - 1) / 100);
    let steps_per_loop = (n / n_proportion) as isize * scaled_step_percentage.signum();
    unsafe {
        worker::STEPS_PER_LOOP = steps_per_loop;
    }
}
