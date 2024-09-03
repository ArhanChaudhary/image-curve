use crate::renderer;
use std::{ptr, thread};

pub static mut STOP_WORKER_LOOP: bool = false;
pub static mut STEPS_PER_LOOP: isize = 1;
pub static mut SLEEP_PER_LOOP: u64 = 0;

use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(js_name = handleWorkerMessage)]
pub fn handle_worker_message(message: JsValue) {
    let received_worker_message: ReceivedWorkerMessage =
        serde_wasm_bindgen::from_value(message).unwrap();
    received_worker_message.process();
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "action", content = "payload")]
pub enum ReceivedWorkerMessage {
    Start,
}

#[derive(Serialize, Deserialize)]
struct LoadImageMessage {
    width: u32,
    height: u32,
    pixel_data: Vec<u8>,
}

impl ReceivedWorkerMessage {
    pub fn process(self) {
        match self {
            Self::Start => {
                start();
            }
        }
    }
}

pub fn start() {
    loop {
        step();
        if unsafe { SLEEP_PER_LOOP } != 0 {
            thread::sleep(std::time::Duration::from_micros(unsafe { SLEEP_PER_LOOP }));
        }
        unsafe {
            if ptr::read_volatile(ptr::addr_of!(STOP_WORKER_LOOP)) {
                STOP_WORKER_LOOP = false;
                break;
            }
        }
    }
}

pub fn step() {
    let width = unsafe { renderer::WIDTH.unwrap() };
    let height = unsafe { renderer::HEIGHT.unwrap() };
    let curve = unsafe { renderer::CURVE.as_mut().unwrap().as_mut_ptr() };
    let pixel_data = unsafe { renderer::PIXEL_DATA.as_mut().unwrap().as_mut_ptr() };

    let n = width * height;
    unsafe {
        if STEPS_PER_LOOP > 0 {
            let step = STEPS_PER_LOOP as usize;
            for curve_index in 0..(n - step) {
                let pixel_index = *curve.add(curve_index);
                let prev_pixel_index = *curve.add((curve_index + n - step) % n);

                core::ptr::swap(
                    pixel_data.add(pixel_index),
                    pixel_data.add(prev_pixel_index),
                );
                core::ptr::swap(
                    pixel_data.add(pixel_index + 1),
                    pixel_data.add(prev_pixel_index + 1),
                );
                core::ptr::swap(
                    pixel_data.add(pixel_index + 2),
                    pixel_data.add(prev_pixel_index + 2),
                );
            }
        } else {
            let step = STEPS_PER_LOOP.unsigned_abs();
            for curve_index in (step..n).rev() {
                let pixel_index = *curve.add(curve_index);
                let prev_pixel_index = *curve.add((curve_index + n - step) % n);

                core::ptr::swap(
                    pixel_data.add(pixel_index),
                    pixel_data.add(prev_pixel_index),
                );
                core::ptr::swap(
                    pixel_data.add(pixel_index + 1),
                    pixel_data.add(prev_pixel_index + 1),
                );
                core::ptr::swap(
                    pixel_data.add(pixel_index + 2),
                    pixel_data.add(prev_pixel_index + 2),
                );
            }
        }
    }
}
