use crate::renderer::{CURVE, HEIGHT, PIXEL_DATA, WIDTH};

pub fn step() {
    let width = unsafe { WIDTH };
    let height = unsafe { HEIGHT };
    let curve = unsafe { CURVE.as_mut_ptr() };
    let pixel_data = unsafe { PIXEL_DATA.as_mut().unwrap().as_mut_ptr() };

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
}
