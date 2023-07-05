mod utils;

use std::rc::Rc;

use views::MyView;
use wasm_bindgen::prelude::*;

mod views;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
    fn setInterval(closure: &Closure<dyn FnMut()>, time: u32) -> i32;
    // Use `js_namespace` here to bind `console.log(..)` instead of just
    // `log(..)`
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen]
pub fn callback() {}

#[wasm_bindgen(start)]
fn run() -> Result<(), JsValue> {
    utils::set_panic_hook();
    // Use `web_sys`'s global `window` function to get a handle on the global
    // window object.
    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");
    let body = document.body().expect("document should have a body");

    let p = Rc::new(MyView::new());
    let _self = Rc::clone(&p);
    p.on_init(Rc::new(document), &body)?;

    Ok(())
}
