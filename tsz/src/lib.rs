mod refs;
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

pub use refs::*;

pub mod format;

pub mod html;

pub use tsz_macros::*;

#[doc(hidden)]
#[macro_export]
macro_rules! __util_format_args {
    ($($args:tt)*) => {
        format_args!($($args)*)
    };
}

pub struct StateRefMut<'a, T>(pub &'a InnerState<T>);

macro_rules! impl_op {
    ($tr:ident, $name:ident) => {
        impl<T: std::ops::$tr<Output = T> + Copy> StateRefMut<'_, T> {
            pub fn $name(self, rhs: T) {
                let new_value = self.0.value.get().$name(rhs);
                self.0.value.set(new_value);

                self.0.publish();
            }
        }
    };
}

impl_op!(Add, add);
impl_op!(Sub, sub);
impl_op!(Mul, mul);
impl_op!(Div, div);
impl_op!(Rem, rem);
impl_op!(BitAnd, bitand);
impl_op!(BitOr, bitor);
impl_op!(BitXor, bitxor);
impl_op!(Shl, shl);
impl_op!(Shr, shr);

impl<'a, T: std::ops::Add<Output = T> + Copy> std::ops::AddAssign<T> for StateRefMut<'a, T> {
    fn add_assign(&mut self, rhs: T) {
        let new_value = self.0.value.get() + rhs;
        self.0.value.set(new_value);

        self.0.publish();
    }
}

pub struct InnerState<T> {
    pub value: Cell<T>,
    subscribers: RefCell<Vec<Box<dyn FnMut(&T)>>>,
}

impl<T: Copy> InnerState<T> {
    pub fn subscribe(&self, f: impl FnMut(&T) + 'static) {
        let mut subs = self.subscribers.borrow_mut();
        subs.push(Box::new(f));
    }

    pub fn publish(&self) {
        let mut subs = self.subscribers.borrow_mut();

        for sub in subs.iter_mut() {
            let value = self.value.get();
            sub(&value)
        }
    }

    pub fn value(&self) -> T {
        self.value.get()
    }

    pub fn value_mut(&self) -> StateRefMut<T> {
        StateRefMut(self)
    }

    // pub fn set_value(&self, new_val: T) {
    //     *self.value.borrow_mut() = new_val;
    //     self.publish();
    // }
}

impl<T> From<T> for InnerState<T> {
    fn from(value: T) -> Self {
        InnerState {
            value: Cell::new(value),
            subscribers: RefCell::new(Vec::new()),
        }
    }
}

pub struct State<T>(Rc<InnerState<T>>);

impl<T> State<T> {
    pub fn bind(&self) -> Binding<T> {
        Binding(self.0.clone())
    }
}

impl<T> std::ops::Deref for State<T> {
    type Target = InnerState<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> From<T> for State<T> {
    fn from(value: T) -> Self {
        State(Rc::new(value.into()))
    }
}

pub struct Binding<T>(Rc<InnerState<T>>);

impl<T> std::ops::Deref for Binding<T> {
    type Target = InnerState<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}