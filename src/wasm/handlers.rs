use crate::messaging::ReceivedWorkerMessage;
use crate::{renderer, utils};
use js_sys::Function;
use std::cell::{Cell, RefCell};
use std::mem;
use std::rc::Rc;
use wasm_bindgen::prelude::Closure;
use wasm_bindgen::JsCast;
use web_sys::{
    CanvasRenderingContext2d, HtmlCanvasElement, HtmlImageElement, HtmlInputElement, Worker,
};

pub async fn uploaded_image(
    upload_input: Rc<HtmlInputElement>,
    canvas: Rc<HtmlCanvasElement>,
    ctx: Rc<CanvasRenderingContext2d>,
) {
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
    pub id: Cell<i32>,
    pub handle: RefCell<Option<Rc<Closure<dyn FnMut()>>>>,
}

pub fn clicked_start(
    ctx: Rc<CanvasRenderingContext2d>,
    worker: Rc<Worker>,
    raf_handle: Rc<RequestAnimationFrameHandle>,
) -> Rc<Closure<dyn FnMut()>> {
    worker
        .post_message(&serde_wasm_bindgen::to_value(&ReceivedWorkerMessage::Start).unwrap())
        .unwrap();

    let raf_handle_clone = raf_handle.clone();
    let render_pixel_data_loop = Rc::new_cyclic(|this| {
        let this = this.clone();
        Closure::<dyn FnMut()>::new(move || {
            renderer::render_pixel_data(ctx.clone());
            let this = this.upgrade().unwrap();
            raf_handle_clone
                .id
                .set(utils::request_animation_frame(&this));
        })
    });
    raf_handle
        .id
        .set(utils::request_animation_frame(&render_pixel_data_loop));
    render_pixel_data_loop
}

pub fn clicked_stop(
    ctx: Rc<CanvasRenderingContext2d>,
    raf_handle: Rc<RequestAnimationFrameHandle>,
) {
    renderer::stop(ctx);
    utils::cancel_animation_frame(raf_handle.id.get());
    mem::forget(raf_handle.handle.borrow_mut().take().unwrap());
}
