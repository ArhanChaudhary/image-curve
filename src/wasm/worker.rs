use crate::renderer;
use std::ptr;

pub static mut STOP_WORKER_LOOP: bool = false;

pub fn start() {
    loop {
        step();
        unsafe {
            if ptr::read_volatile(ptr::addr_of!(STOP_WORKER_LOOP)) {
                STOP_WORKER_LOOP = false;
                break;
            }
        }
    }
}

pub fn step() {
    let width = unsafe { renderer::WIDTH };
    let height = unsafe { renderer::HEIGHT };
    let curve = unsafe { renderer::CURVE.as_mut_ptr() };
    let pixel_data = unsafe { renderer::PIXEL_DATA.as_mut().unwrap().as_mut_ptr() };

    let n = width * height;
    let step = 1;
    unsafe {
        for curve_index in 0..(n - step) {
            let pixel_index = *curve.add(curve_index);
            let prev_pixel_index = *curve.add((curve_index + n - step) % n);

            let temp_r = *pixel_data.add(pixel_index);
            *pixel_data.add(pixel_index) = *pixel_data.add(prev_pixel_index);
            *pixel_data.add(prev_pixel_index) = temp_r;

            let temp_g = *pixel_data.add(pixel_index + 1);
            *pixel_data.add(pixel_index + 1) = *pixel_data.add(prev_pixel_index + 1);
            *pixel_data.add(prev_pixel_index + 1) = temp_g;

            let temp_b = *pixel_data.add(pixel_index + 2);
            *pixel_data.add(pixel_index + 2) = *pixel_data.add(prev_pixel_index + 2);
            *pixel_data.add(prev_pixel_index + 2) = temp_b;
        }
    }
}
