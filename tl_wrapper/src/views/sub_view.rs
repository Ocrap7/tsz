use tl_util::Binding;

pub struct SubView {
    sub_value: Binding<u64>,
}

impl SubView {
    pub fn new(value_param: Binding<u64>) -> Self {
        Self {
            sub_value: value_param,
        }
    }
}

tl_core::view! {
    declare SubView;

    div {
        "SubView test {$sub_value}"
    }
}
