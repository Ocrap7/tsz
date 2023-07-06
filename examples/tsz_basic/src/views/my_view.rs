use super::sub_view::*;

pub struct MyView {
    value: State<u64>,
}

impl MyView {
    pub fn new() -> MyView {
        MyView { value: 0.into() }
    }

    pub fn inc(self: Rc<Self>) {
        self.value.value_mut().add(2)
    }
}

tsz::view! {
    declare MyView;

    div (class: [container]) {
        "The count is {$value}"

        div (class: [center]) {
            button (click: { $value += 1 }) {
                "Count"
            }
        }
    }

    div {
        SubView($value);
    }
}
