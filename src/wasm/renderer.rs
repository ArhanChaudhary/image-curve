use crate::{handlers, paths, utils, worker, GlobalState};
use js_sys::{Uint8ClampedArray, WebAssembly};
use std::sync::atomic::Ordering;
use wasm_bindgen::prelude::*;

#[derive(Copy, Clone, Debug, Default)]
pub struct ImageDimensions {
    width: u32,
    height: u32,
}

pub struct Point(pub i32, pub i32);

pub async fn load_image(global_state: &GlobalState) {
    let width = global_state.ctx.canvas().unwrap().width();
    let height = global_state.ctx.canvas().unwrap().height();
    let pixel_data = global_state
        .ctx
        .get_image_data(0.0, 0.0, width as f64, height as f64)
        .unwrap()
        .data()
        .0;
    *worker::PIXEL_DATA.lock().unwrap() = pixel_data;

    *global_state.image_dimensions.borrow_mut() = ImageDimensions { width, height };
    let received_worker_message = utils::worker_operation(
        &global_state.worker,
        worker::WorkerMessage::LoadPath(worker::LoadPathMessage::new(
            width,
            height,
            paths::gilbert_d2xy,
        )),
    )
    .await;
    let handlers::MainMessage::LoadedPath { path_len } = received_worker_message else {
        panic!(
            "Expected MainMessage::LoadedPath, got {:?}",
            received_worker_message
        );
    };
    global_state.path_len.set(Some(path_len));
}

#[wasm_bindgen]
extern "C" {
    #[derive(Debug)]
    type ImageData;

    #[wasm_bindgen(constructor, catch)]
    fn new(data: &Uint8ClampedArray, width: u32, height: u32) -> Result<ImageData, JsValue>;
}

pub fn render_pixel_data(global_state: &GlobalState) {
    let pixel_data = worker::PIXEL_DATA.lock().unwrap();
    let pixel_data_base = pixel_data.as_ptr() as usize;
    let pixel_data_len = pixel_data.len() as u32;
    let sliced_pixel_data = Uint8ClampedArray::new(
        &wasm_bindgen::memory()
            .unchecked_into::<WebAssembly::Memory>()
            .buffer(),
    )
    .slice(
        pixel_data_base as u32,
        pixel_data_base as u32 + pixel_data_len,
    );

    let image_dimensions = global_state.image_dimensions.borrow();
    let image_data = &ImageData::new(
        &sliced_pixel_data,
        image_dimensions.width,
        image_dimensions.height,
    )
    .unwrap()
    .dyn_into::<web_sys::ImageData>()
    .unwrap();

    global_state
        .ctx
        .put_image_data(image_data, 0.0, 0.0)
        .unwrap();
}

pub async fn stop(global_state: &GlobalState) {
    worker::STOP_WORKER_LOOP.store(true, Ordering::Relaxed);
    let received_worker_message = utils::wait_for_worker_message(&global_state.worker).await;
    if received_worker_message != handlers::MainMessage::Stopped {
        panic!(
            "Expected MainMessage::Stopped, got {:?}",
            received_worker_message
        );
    };
    render_pixel_data(global_state);
}

const ALL_SLEEPS_PER_LOOP: [u32; 10] = [200_000, 175_000, 50_000, 10_000, 2500, 500, 40, 20, 10, 0];

pub fn change_speed(new_speed_percentage: u32) {
    let lerped: u64 = utils::lerp(&ALL_SLEEPS_PER_LOOP, new_speed_percentage);
    worker::SLEEP.store(lerped, Ordering::Relaxed);
}

pub fn change_step(new_step_percentage: u32, global_state: &GlobalState) {
    let scaled_step_percentage = ((new_step_percentage as i32 - 50) * 2) as f64;
    if scaled_step_percentage == 0.0 {
        worker::STEPS.store(0, Ordering::Relaxed);
        return;
    }
    let path_len = global_state.path_len.get().unwrap() as f64;
    let log_proportion =
        path_len.log2() - scaled_step_percentage.abs() * (path_len.log2() - 1.0) / 100.0;
    let steps =
        (path_len / 2.0_f64.powf(log_proportion)) as i32 * scaled_step_percentage.signum() as i32;
    worker::STEPS.store(steps, Ordering::Relaxed);
}
