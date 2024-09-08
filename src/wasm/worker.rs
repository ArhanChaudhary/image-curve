use crate::{handlers, renderer};
use serde::{Deserialize, Serialize};
use std::{cell::RefCell, ptr, thread};
use wasm_bindgen::prelude::*;
use web_sys::{DedicatedWorkerGlobalScope, MessageEvent};

pub static mut STOP_WORKER_LOOP: bool = false;
pub static mut STEPS: i32 = 1;
pub static mut SLEEP: u64 = 0;

#[derive(Default)]
struct GlobalState {
    path: RefCell<Vec<usize>>,
}

#[wasm_bindgen(js_name = runWorker)]
pub fn run_worker() {
    let global_state: GlobalState = Default::default();
    let closure = Closure::<dyn Fn(_)>::new(move |e: MessageEvent| {
        let message = e.data();
        let received_worker_message: WorkerMessage =
            serde_wasm_bindgen::from_value(message).unwrap();
        received_worker_message.process(&global_state);
    });
    js_sys::global()
        .unchecked_into::<DedicatedWorkerGlobalScope>()
        .set_onmessage(Some(closure.as_ref().unchecked_ref()));
    closure.forget();
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "action", content = "payload")]
pub enum WorkerMessage {
    Start,
    Step,
    LoadPath(LoadPathMessage),
}

#[derive(Serialize, Deserialize)]
pub struct LoadPathMessage {
    width: u32,
    height: u32,
    path_fn_ptr: usize,
}

type PathFn = fn(u32, u32, u32) -> renderer::Point;

impl LoadPathMessage {
    pub fn new(width: u32, height: u32, path_fn: PathFn) -> Self {
        Self {
            width,
            height,
            path_fn_ptr: path_fn as usize,
        }
    }
}

impl WorkerMessage {
    fn process(self, global_state: &GlobalState) {
        match self {
            Self::Start => {
                start(global_state);
                js_sys::global()
                    .unchecked_into::<DedicatedWorkerGlobalScope>()
                    .post_message(
                        &serde_wasm_bindgen::to_value(&handlers::MainMessage::Stopped).unwrap(),
                    )
                    .unwrap();
            }
            Self::Step => {
                step(global_state);
                js_sys::global()
                    .unchecked_into::<DedicatedWorkerGlobalScope>()
                    .post_message(
                        &serde_wasm_bindgen::to_value(&handlers::MainMessage::Stepped).unwrap(),
                    )
                    .unwrap();
            }
            Self::LoadPath(load_path_message) => {
                let path_len = load_path(load_path_message, global_state);
                js_sys::global()
                    .unchecked_into::<DedicatedWorkerGlobalScope>()
                    .post_message(
                        &serde_wasm_bindgen::to_value(&handlers::MainMessage::LoadedPath {
                            path_len,
                        })
                        .unwrap(),
                    )
                    .unwrap();
            }
        }
    }
}

fn start(global_state: &GlobalState) {
    loop {
        step(global_state);
        let sleep = unsafe { SLEEP };
        if sleep != 0 {
            thread::sleep(std::time::Duration::from_micros(sleep));
        }
        if unsafe { ptr::read_volatile(ptr::addr_of!(STOP_WORKER_LOOP)) } {
            unsafe {
                STOP_WORKER_LOOP = false;
            }
            break;
        }
    }
}

fn step(global_state: &GlobalState) {
    let mut path = global_state.path.borrow_mut();
    let path_len = path.len();
    let path_ptr = path.as_mut_ptr();
    let pixel_data_ptr = unsafe { renderer::PIXEL_DATA.as_mut_ptr() };
    let steps = unsafe { STEPS } as isize;
    if steps > 0 {
        let steps = steps as usize;
        for path_index in 0..(path_len - steps) {
            unsafe {
                swap_pixel(
                    path_ptr.add(path_index),
                    path_ptr.add((path_index + path_len - steps) % path_len),
                    pixel_data_ptr,
                );
            }
        }
    } else {
        let steps = steps.unsigned_abs();
        for path_index in (steps..path_len).rev() {
            unsafe {
                swap_pixel(
                    path_ptr.add(path_index),
                    path_ptr.add((path_index + path_len - steps) % path_len),
                    pixel_data_ptr,
                );
            }
        }
    }
}

unsafe fn swap_pixel(
    first_pixel_ptr: *mut usize,
    second_pixel_ptr: *mut usize,
    pixel_data_ptr: *mut u8,
) {
    core::ptr::swap(
        pixel_data_ptr.add(*first_pixel_ptr),
        pixel_data_ptr.add(*second_pixel_ptr),
    );
    core::ptr::swap(
        pixel_data_ptr.add(*first_pixel_ptr + 1),
        pixel_data_ptr.add(*second_pixel_ptr + 1),
    );
    core::ptr::swap(
        pixel_data_ptr.add(*first_pixel_ptr + 2),
        pixel_data_ptr.add(*second_pixel_ptr + 2),
    );
}

fn load_path(load_path_message: LoadPathMessage, global_state: &GlobalState) -> u32 {
    let path_fn: PathFn = unsafe { std::mem::transmute(load_path_message.path_fn_ptr) };
    let mut path: Vec<_> = (0..(load_path_message.width * load_path_message.height))
        .map(|idx| path_fn(idx, load_path_message.width, load_path_message.height))
        .map(|renderer::Point(x, y)| {
            (y.rem_euclid(load_path_message.height as i32) as usize
                * load_path_message.width as usize
                + x.rem_euclid(load_path_message.width as i32) as usize)
                * 4
        })
        .collect();
    path.dedup();
    let path_len = path.len();
    *global_state.path.borrow_mut() = path;
    path_len as u32
}
