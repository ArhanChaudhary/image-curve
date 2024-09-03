use crate::{renderer, utils, worker};
use js_sys::Function;
use std::cell::RefCell;
use std::mem;
use std::rc::Rc;
use wasm_bindgen::prelude::Closure;
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlImageElement, HtmlInputElement, PointerEvent, Worker};

pub async fn uploaded_image(upload_input: Rc<HtmlInputElement>, ctx: Rc<CanvasRenderingContext2d>) {
    let src = crate::utils::to_base64(upload_input.files().unwrap().get(0).unwrap()).await;
    let img = HtmlImageElement::new().unwrap();
    img.set_src(&src);
    wasm_bindgen_futures::JsFuture::from(
        utils::PromiseOnlyResolve::new(&mut |resolve: Function| {
            img.set_onload(Some(&resolve));
        })
        .dyn_into::<js_sys::Promise>()
        .unwrap(),
    )
    .await
    .unwrap();
    let width = img.width();
    let height = img.height();
    let canvas = ctx.canvas().unwrap();
    canvas.set_width(width);
    canvas.set_height(height);
    ctx.draw_image_with_html_image_element_and_dw_and_dh(
        &img,
        0.0,
        0.0,
        width as f64,
        height as f64,
    )
    .unwrap();
    renderer::load_image(ctx);
    // change speed / step here TODO:
}

pub struct RequestAnimationFrameHandle {
    pub id: i32,
    pub _closure: Rc<Closure<dyn FnMut()>>,
}

pub fn clicked_start(
    ctx: Rc<CanvasRenderingContext2d>,
    worker: Rc<Worker>,
    raf_handle: Rc<RefCell<Option<RequestAnimationFrameHandle>>>,
) {
    worker
        .post_message(&serde_wasm_bindgen::to_value(&worker::ReceivedWorkerMessage::Start).unwrap())
        .unwrap();

    let raf_handle_clone = raf_handle.clone();
    let render_pixel_data_loop = Rc::new_cyclic(|this| {
        let this = this.clone();
        Closure::<dyn FnMut()>::new(move || {
            renderer::render_pixel_data(ctx.clone());
            let this = this.upgrade().unwrap();
            raf_handle_clone.borrow_mut().as_mut().unwrap().id =
                utils::request_animation_frame(&this);
        })
    });
    *raf_handle.borrow_mut() = Some(RequestAnimationFrameHandle {
        id: utils::request_animation_frame(&render_pixel_data_loop),
        _closure: render_pixel_data_loop,
    });
}

pub fn clicked_stop(
    ctx: Rc<CanvasRenderingContext2d>,
    raf_handle: Rc<RefCell<Option<RequestAnimationFrameHandle>>>,
) {
    renderer::stop(ctx);
    let taken = raf_handle.borrow_mut().take().unwrap();
    utils::cancel_animation_frame(taken.id);
    mem::drop(taken);
}

pub fn clicked_step(ctx: Rc<CanvasRenderingContext2d>) {
    worker::step();
    renderer::render_pixel_data(ctx);
}

pub fn inputted_speed(e: PointerEvent) {
    let new_speed_percentage = e
        .target()
        .unwrap()
        .unchecked_into::<HtmlInputElement>()
        .value_as_number() as usize;
    renderer::change_speed(new_speed_percentage)
}

pub fn inputted_step(e: PointerEvent) {
    let change_step_message = e
        .target()
        .unwrap()
        .unchecked_into::<HtmlInputElement>()
        .value_as_number() as usize;
    renderer::change_step(change_step_message)
}
