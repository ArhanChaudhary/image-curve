use crate::{handlers, renderer};
use std::{ptr, thread};

pub static mut STOP_WORKER_LOOP: bool = false;
pub static mut STEPS: isize = 1;
pub static mut SLEEP: u64 = 0;

use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use web_sys::DedicatedWorkerGlobalScope;

#[wasm_bindgen(js_name = handleWorkerMessage)]
pub fn handle_worker_message(message: JsValue) {
    let received_worker_message: WorkerMessage = serde_wasm_bindgen::from_value(message).unwrap();
    received_worker_message.process();
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "action", content = "payload")]
pub enum WorkerMessage {
    Start,
    Step,
}

#[derive(Serialize, Deserialize)]
struct LoadImageMessage {
    pixel_data: Vec<u8>,
}

impl WorkerMessage {
    fn process(self) {
        match self {
            Self::Start => {
                start();
            }
            Self::Step => {
                step();
                js_sys::global()
                    .unchecked_into::<DedicatedWorkerGlobalScope>()
                    .post_message(
                        &serde_wasm_bindgen::to_value(&handlers::MainMessage::Stepped).unwrap(),
                    )
                    .unwrap();
            }
        }
    }
}

fn start() {
    loop {
        step();
        let sleep = unsafe { SLEEP };
        if sleep != 0 {
            thread::sleep(std::time::Duration::from_micros(sleep));
        }
        unsafe {
            if ptr::read_volatile(ptr::addr_of!(STOP_WORKER_LOOP)) {
                STOP_WORKER_LOOP = false;
                break;
            }
        }
    }
}

fn step() {
    let curve = unsafe { renderer::CURVE.as_mut().unwrap() };
    let pixel_data = unsafe { renderer::PIXEL_DATA.as_mut().unwrap() };
    let curve_len = curve.len();
    let steps = unsafe { STEPS };
    if steps > 0 {
        let steps = steps as usize;
        for curve_index in 0..(curve_len - steps) {
            unsafe {
                swap_pixel(
                    curve.as_mut_ptr().add(curve_index),
                    curve.as_mut_ptr().add((curve_index + curve_len - steps) % curve_len),
                    pixel_data.as_mut_ptr(),
                );
            }
        }
    } else {
        let steps = steps.unsigned_abs();
        for curve_index in (steps..curve_len).rev() {
            unsafe {
                swap_pixel(
                    curve.as_mut_ptr().add(curve_index),
                    curve.as_mut_ptr().add((curve_index + curve_len - steps) % curve_len),
                    pixel_data.as_mut_ptr(),
                );
            }
        }
    }
}

unsafe fn swap_pixel(
    first_pixel_ptr: *mut usize,
    second_pixel_ptr: *mut usize,
    pixel_data: *mut u8,
) {
    core::ptr::swap(
        pixel_data.add(*first_pixel_ptr),
        pixel_data.add(*second_pixel_ptr),
    );
    core::ptr::swap(
        pixel_data.add(*first_pixel_ptr + 1),
        pixel_data.add(*second_pixel_ptr + 1),
    );
    core::ptr::swap(
        pixel_data.add(*first_pixel_ptr + 2),
        pixel_data.add(*second_pixel_ptr + 2),
    );
}
