use crate::{self as tsz, Binding};

enum BindingOrIter<T> {
    Binding(Binding<T>),
    Iter(Vec<T>)
}

pub struct For<T> {
    elements: BindingOrIter<T>,
}

crate::view! {
    declare <T> For<T>;
}