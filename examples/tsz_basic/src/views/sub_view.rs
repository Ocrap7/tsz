use tsz::Binding;

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

tsz::view! {
    declare SubView;

    div {
        #children
        "SubView test {$sub_value}"
    }
}
