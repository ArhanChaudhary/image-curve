use crate::{renderer, utils};
use js_sys::Function;
use std::rc::Rc;
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, HtmlImageElement, HtmlInputElement, Worker};

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

pub fn clicked_start(worker: Rc<Worker>) {
    worker
        .post_message(&wasm_bindgen::module())
        .unwrap();

}
