mod utils;

use std::rc::Rc;

use tl_util::State;
use wasm_bindgen::prelude::*;
use web_sys::Event;

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
        pub fn inc(self: Rc<Self>) {
            self.value.value_mut().add(1);
        }
    }

    tl_core::load! {
        declare Potato;

        div (class: [container]) {
            "Hello {$value}"
        }

        button (click: { log("Hello") }) {
            "Count"
        }
    };

    // let e = document.create_element("button")?;
    // e.set_text_content(Some("bruh"));
    // let cb: Closure<dyn FnMut(Event)> = Closure::new(move |_| {
    //     log("Hi");
    // });
    // e.add_event_listener_with_callback("click", &cb.as_ref().unchecked_ref())?;
    // body.append_child(&e)?;
    // let n = document.create_text_node("datfjdsklfa");
    // // n.clone();
    // cb.forget();

    let p = Rc::new(Potato { value: 0.into() });
    let _self = Rc::clone(&p);
    p.on_init(Rc::new(document), &body)?;

    Ok(())
}
