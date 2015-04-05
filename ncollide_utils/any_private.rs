use std::marker::Reflect;
use std::any::TypeId;

pub trait AnyPrivate {
    /// The type id of `Self`.
    ///
    /// NOTE: this exists only because `Any::get_type_id()` is private for some reason…
    fn get_dyn_type_id(&self) -> TypeId;
}

impl<T: 'static> AnyPrivate for T 
    where T: Reflect
{
    #[inline]
    fn get_dyn_type_id(&self) -> TypeId {
        TypeId::of::<T>()
    }
}

