use super::sub_view::*;
use tsz::views::*;

pub struct MyView {
    value: State<bool>,
}

impl MyView {
    pub fn new() -> MyView {
        MyView { value: false.into() }
    }

}

tsz::view! {
    declare MyView;

    div (class: [container]) {
        "The count is {$value}"

        div (class: [center]) {
            button (click: { $value = true }) {
                "Count"
            }
        }
    }

    If($value) {
        "Is true"
    }
}
