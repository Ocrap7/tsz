mod utils;

use std::{rc::Rc};

use tl_util::State;
use wasm_bindgen::prelude::*;

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
    // let e = body.insert_adjacent_element(where_, element)?.unwrap();
    // let tn = document.create_text_node("fsdf");
    // body.append_child(&tn.get_root_node());
    // tn.
    // document.insert_adja
    // body.

    struct Potato {
        value: State<u64>,
    }

    impl Potato {
        pub fn p(self: Rc<Self>) {}
    }

    tl_core::load! {
        declare Potato;

        div (class: [container]) {
            "Hello {$value}"
        }
    };

    let p = Rc::new(Potato { value: 0.into() });
    let rp = p.clone();
    p.on_init(Rc::new(document), &body)?;

    let cb = Closure::new(move || {
        rp.value.value_mut().add(1);
        rp.value.value_mut().rem(5);
    });

    setInterval(&cb, 500);

    cb.forget();

    Ok(())
}
