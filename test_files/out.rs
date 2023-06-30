use tl_core::html::*;

struct ViewName {

}

impl ViewName {
    pub fn on_init(document: &Document, parent: &Element) -> Result<(), JsValue> {
        let _e0 = document.create_element("div");
        _e0.set_text_content(Some("Hello"));

        parent.append_child($_e0);

        Ok(())
    }
}