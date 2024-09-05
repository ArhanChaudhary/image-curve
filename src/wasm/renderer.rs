use crate::{gilbert, worker};
use js_sys::{Uint8ClampedArray, WebAssembly};
use std::{cell::OnceCell, ptr, rc::Rc};
use wasm_bindgen::prelude::*;
use web_sys::CanvasRenderingContext2d;

pub static mut CURVE: Option<Vec<usize>> = None;
pub static mut PIXEL_DATA: Option<Vec<u8>> = None;

#[derive(Copy, Clone, Debug)]
pub struct ImageDimensions {
    width: usize,
    height: usize,
}

pub fn load_image(
    ctx: Rc<CanvasRenderingContext2d>,
    image_dimensions: Rc<OnceCell<ImageDimensions>>,
) {
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
        CURVE = Some(curve);
        PIXEL_DATA = Some(pixel_data);
    }
    image_dimensions
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

pub fn render_pixel_data(
    ctx: Rc<CanvasRenderingContext2d>,
    image_dimensions: Rc<OnceCell<ImageDimensions>>,
) {
    let pixel_data = unsafe { PIXEL_DATA.as_ref().unwrap() };
    let base = pixel_data.as_ptr() as u32;
    let len = pixel_data.len() as u32;
    let sliced_pixel_data = Uint8ClampedArray::new(
        &wasm_bindgen::memory()
            .unchecked_into::<WebAssembly::Memory>()
            .buffer(),
    )
    .slice(base, base + len);

    let image_data = &ImageData::new(
        &sliced_pixel_data,
        image_dimensions.get().unwrap().width as u32,
        image_dimensions.get().unwrap().height as u32,
    )
    .unwrap()
    .dyn_into::<web_sys::ImageData>()
    .unwrap();

    ctx.put_image_data(image_data, 0.0, 0.0).unwrap();
}

pub fn stop(ctx: Rc<CanvasRenderingContext2d>, image_dimensions: Rc<OnceCell<ImageDimensions>>) {
    unsafe {
        worker::STOP_WORKER_LOOP = true;
        while ptr::read_volatile(ptr::addr_of!(worker::STOP_WORKER_LOOP)) {}
    }
    render_pixel_data(ctx, image_dimensions);
}

const ALL_SLEEPS_PER_LOOP: [usize; 10] =
    [200_000, 175_000, 50_000, 10_000, 2500, 500, 40, 20, 10, 0];

pub fn change_speed(new_speed_percentage: usize) {
    let lerped: u64 = crate::utils::lerp(ALL_SLEEPS_PER_LOOP, new_speed_percentage);
    unsafe {
        worker::SLEEP = lerped;
    }
}

pub fn change_step(new_step_percentage: usize, image_dimensions: Rc<OnceCell<ImageDimensions>>) {
    let scaled_step_percentage = (new_step_percentage as isize - 50) * 2;
    let ImageDimensions { width, height } = image_dimensions.get().unwrap();
    let curve_len = width * height;
    let curve_len_proportion = 2_usize.pow(
        curve_len.ilog2()
            - scaled_step_percentage.unsigned_abs() as u32 * (curve_len.ilog2() - 1) / 100,
    );
    let steps = (curve_len / curve_len_proportion) as isize * scaled_step_percentage.signum();
    unsafe {
        worker::STEPS = steps;
    }
}
