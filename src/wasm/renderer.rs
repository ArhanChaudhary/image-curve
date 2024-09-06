use crate::{paths, worker, GlobalState};
use js_sys::{Uint8ClampedArray, WebAssembly};
use std::{ptr, rc::Rc};
use wasm_bindgen::prelude::*;

pub static mut PATH: Option<Vec<usize>> = None;
pub static mut PIXEL_DATA: Option<Vec<u8>> = None;

#[derive(Copy, Clone, Debug)]
pub struct ImageDimensions {
    width: u32,
    height: u32,
}

pub struct Point(pub i32, pub i32);

pub fn load_image(global_state: Rc<GlobalState>) {
    let width = global_state.ctx.canvas().unwrap().width();
    let height = global_state.ctx.canvas().unwrap().height();
    let pixel_data = global_state
        .ctx
        .get_image_data(0.0, 0.0, width as f64, height as f64)
        .unwrap()
        .data()
        .0;
    let mut path: Vec<_> = (0..(width * height))
        .map(|idx| paths::shift(idx, width, height))
        .map(|Point(x, y)| {
            (y.rem_euclid(height as i32) as usize * width as usize
                + x.rem_euclid(width as i32) as usize)
                * 4
        })
        .collect();
    path.dedup();
    global_state.path_len.set(Some(path.len() as u32));
    unsafe {
        PATH = Some(path);
        PIXEL_DATA = Some(pixel_data);
    }
    global_state
        .image_dimensions
        .set(ImageDimensions { width, height })
        .unwrap();
}

#[wasm_bindgen]
extern "C" {
    #[derive(Debug)]
    type ImageData;

    #[wasm_bindgen(constructor, catch)]
    fn new(data: &Uint8ClampedArray, width: u32, height: u32) -> Result<ImageData, JsValue>;
}

pub fn render_pixel_data(global_state: Rc<GlobalState>) {
    let pixel_data = unsafe { PIXEL_DATA.as_ref().unwrap() };
    let pixel_data_base = pixel_data.as_ptr() as u32;
    let pixel_data_len = pixel_data.len() as u32;
    let sliced_pixel_data = Uint8ClampedArray::new(
        &wasm_bindgen::memory()
            .unchecked_into::<WebAssembly::Memory>()
            .buffer(),
    )
    .slice(pixel_data_base, pixel_data_base + pixel_data_len);

    let image_data = &ImageData::new(
        &sliced_pixel_data,
        global_state.image_dimensions.get().unwrap().width,
        global_state.image_dimensions.get().unwrap().height,
    )
    .unwrap()
    .dyn_into::<web_sys::ImageData>()
    .unwrap();

    global_state
        .ctx
        .put_image_data(image_data, 0.0, 0.0)
        .unwrap();
}

pub fn stop(global_state: Rc<GlobalState>) {
    unsafe {
        worker::STOP_WORKER_LOOP = true;
        while ptr::read_volatile(ptr::addr_of!(worker::STOP_WORKER_LOOP)) {}
    }
    render_pixel_data(global_state);
}

const ALL_SLEEPS_PER_LOOP: [u32; 10] = [200_000, 175_000, 50_000, 10_000, 2500, 500, 40, 20, 10, 0];

pub fn change_speed(new_speed_percentage: u32) {
    let lerped: u64 = crate::utils::lerp(ALL_SLEEPS_PER_LOOP, new_speed_percentage);
    unsafe {
        worker::SLEEP = lerped;
    }
}

pub fn change_step(new_step_percentage: u32, global_state: Rc<GlobalState>) {
    let scaled_step_percentage = (new_step_percentage as i32 - 50) * 2;
    let path_len = global_state.path_len.get().unwrap();
    let path_len_proportion = 2_u32.pow(
        path_len.ilog2() - scaled_step_percentage.unsigned_abs() * (path_len.ilog2() - 1) / 100,
    );
    let steps = (path_len / path_len_proportion) as i32 * scaled_step_percentage.signum();
    unsafe {
        worker::STEPS = steps;
    }
}
