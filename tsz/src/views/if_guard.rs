use crate::{self as tsz, Binding};

pub struct If {
    condition: Binding<bool>,
}

impl If {
    pub fn new(binding: Binding<bool>) -> If {
        If { condition: binding }
    }
}

use std::rc::Rc;
use wasm_bindgen::prelude::*;

impl If {
    pub fn on_init(
        self: Rc<Self>,
        document: Rc<tsz::html::Document>,
        parent: &tsz::html::Element,
        children: Option<
            fn(&Rc<Self>, &Rc<tsz::html::Document>, &tsz::html::Element) -> Result<tsz::html::Element, JsValue>,
        >,
    ) -> Result<(), JsValue> {
        let __body = document.body().expect("Unable to get document body");
        let _self = self;

        let children = Rc::new(children.expect("Expected children!"));
        // let parent = Rc::new(children.expect("Expected children!"));

        if _self.condition.value() {
            // if let Some(view) = &children {
                children(&_self, &document, &parent)?;
            // }
        }

        let _selfc = _self.clone();
        let _children = children.clone();
        let _doc = document.clone();
        let _parent = parent.clone();

        _self.condition.subscribe(move |value| {
            let _children = Rc::clone(&_children);
            if *value {
                // if let Some(view) = &children {
                _children(&_selfc, &_doc, &_parent).expect("Creating chidlren failed");
                // }
            }
        });

        Ok(())
    }
}
