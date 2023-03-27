use crate::fmt::display;
use crate::kind::Trivial;
use crate::string::CxxString;
use crate::ExternType;
use core::ffi::c_void;
use core::fmt::{self, Debug, Display};
use core::marker::PhantomData;
use core::mem::MaybeUninit;
use core::ops::Deref;

/// Binding to C++ `seastar::shared_ptr<T>`.
#[repr(C)]
pub struct SeastarSharedPtr<T>
where
    T: SeastarSharedPtrTarget,
{
    repr: [MaybeUninit<*mut c_void>; 2],
    ty: PhantomData<T>,
}

impl<T> SeastarSharedPtr<T>
where
    T: SeastarSharedPtrTarget,
{
    /// Makes a new SeastarSharedPtr wrapping a null pointer.
    ///
    /// Matches the behavior of default-constructing a seastar::shared\_ptr.
    pub fn null() -> Self {
        let mut shared_ptr = MaybeUninit::<SeastarSharedPtr<T>>::uninit();
        let new = shared_ptr.as_mut_ptr().cast();
        unsafe {
            T::__null(new);
            shared_ptr.assume_init()
        }
    }

    /// Allocates memory on the heap and makes a SeastarSharedPtr owner for it.
    pub fn new(value: T) -> Self
    where
        T: ExternType<Kind = Trivial>,
    {
        let mut shared_ptr = MaybeUninit::<SeastarSharedPtr<T>>::uninit();
        let new = shared_ptr.as_mut_ptr().cast();
        unsafe {
            T::__new(value, new);
            shared_ptr.assume_init()
        }
    }

    /// Checks whether the SeastarSharedPtr does not own an object.
    ///
    /// This is the opposite of [seastar::shared_ptr\<T\>::operator bool].
    pub fn is_null(&self) -> bool {
        let this = self as *const Self as *const c_void;
        let ptr = unsafe { T::__get(this) };
        ptr.is_null()
    }

    /// Returns a reference to the object owned by this SeastarSharedPtr if any,
    /// otherwise None.
    pub fn as_ref(&self) -> Option<&T> {
        let this = self as *const Self as *const c_void;
        unsafe { T::__get(this).as_ref() }
    }
}

unsafe impl<T> Send for SeastarSharedPtr<T> where T: Send + Sync + SeastarSharedPtrTarget {}
unsafe impl<T> Sync for SeastarSharedPtr<T> where T: Send + Sync + SeastarSharedPtrTarget {}

impl<T> Clone for SeastarSharedPtr<T>
where
    T: SeastarSharedPtrTarget,
{
    fn clone(&self) -> Self {
        let mut shared_ptr = MaybeUninit::<SeastarSharedPtr<T>>::uninit();
        let new = shared_ptr.as_mut_ptr().cast();
        let this = self as *const Self as *mut c_void;
        unsafe {
            T::__clone(this, new);
            shared_ptr.assume_init()
        }
    }
}

// SeastarSharedPtr is not a self-referential type and is safe to move out of a Pin,
// regardless whether the pointer's target is Unpin.
impl<T> Unpin for SeastarSharedPtr<T> where T: SeastarSharedPtrTarget {}

impl<T> Drop for SeastarSharedPtr<T>
where
    T: SeastarSharedPtrTarget,
{
    fn drop(&mut self) {
        let this = self as *mut Self as *mut c_void;
        unsafe { T::__drop(this) }
    }
}

impl<T> Deref for SeastarSharedPtr<T>
where
    T: SeastarSharedPtrTarget,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match self.as_ref() {
            Some(target) => target,
            None => panic!(
                "called deref on a null SeastarSharedPtr<{}>",
                display(T::__typename),
            ),
        }
    }
}

impl<T> Debug for SeastarSharedPtr<T>
where
    T: Debug + SeastarSharedPtrTarget,
{
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self.as_ref() {
            None => formatter.write_str("nullptr"),
            Some(value) => Debug::fmt(value, formatter),
        }
    }
}

impl<T> Display for SeastarSharedPtr<T>
where
    T: Display + SeastarSharedPtrTarget,
{
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self.as_ref() {
            None => formatter.write_str("nullptr"),
            Some(value) => Display::fmt(value, formatter),
        }
    }
}

/// Trait bound for types which may be used as the `T` inside of a
/// `SeastarSharedPtr<T>` in generic code.
///
/// This trait has no publicly callable or implementable methods. Implementing
/// it outside of the CXX codebase is not supported.
///
/// # Example
///
/// A bound `T: SeastarSharedPtrTarget` may be necessary when manipulating
/// [`SeastarSharedPtr`] in generic code.
///
/// ```
/// use cxx::memory::{SeastarSharedPtr, SeastarSharedPtrTarget};
/// use std::fmt::Display;
///
/// pub fn take_generic_ptr<T>(ptr: SeastarSharedPtr<T>)
/// where
///     T: SeastarSharedPtrTarget + Display,
/// {
///     println!("the shared_ptr points to: {}", *ptr);
/// }
/// ```
///
/// Writing the same generic function without a `SeastarSharedPtrTarget` trait bound
/// would not compile.
pub unsafe trait SeastarSharedPtrTarget {
    #[doc(hidden)]
    fn __typename(f: &mut fmt::Formatter) -> fmt::Result;
    #[doc(hidden)]
    unsafe fn __null(new: *mut c_void);
    #[doc(hidden)]
    unsafe fn __new(value: Self, new: *mut c_void)
    where
        Self: Sized,
    {
        // Opoaque C types do not get this method because they can never exist
        // by value on the Rust side of the bridge.
        let _ = value;
        let _ = new;
        unreachable!()
    }
    #[doc(hidden)]
    unsafe fn __clone(this: *const c_void, new: *mut c_void);
    #[doc(hidden)]
    unsafe fn __get(this: *const c_void) -> *const Self;
    #[doc(hidden)]
    unsafe fn __drop(this: *mut c_void);
}

macro_rules! impl_shared_ptr_target {
    ($segment:expr, $name:expr, $ty:ty) => {
        unsafe impl SeastarSharedPtrTarget for $ty {
            fn __typename(f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str($name)
            }
            unsafe fn __null(new: *mut c_void) {
                extern "C" {
                    attr! {
                        #[link_name = concat!("cxxbridge1$seastar$shared_ptr$", $segment, "$null")]
                        fn __null(new: *mut c_void);
                    }
                }
                unsafe { __null(new) }
            }
            unsafe fn __new(value: Self, new: *mut c_void) {
                extern "C" {
                    attr! {
                        #[link_name = concat!("cxxbridge1$seastar$shared_ptr$", $segment, "$uninit")]
                        fn __uninit(new: *mut c_void) -> *mut c_void;
                    }
                }
                unsafe { __uninit(new).cast::<$ty>().write(value) }
            }
            unsafe fn __clone(this: *const c_void, new: *mut c_void) {
                extern "C" {
                    attr! {
                        #[link_name = concat!("cxxbridge1$seastar$shared_ptr$", $segment, "$clone")]
                        fn __clone(this: *const c_void, new: *mut c_void);
                    }
                }
                unsafe { __clone(this, new) }
            }
            unsafe fn __get(this: *const c_void) -> *const Self {
                extern "C" {
                    attr! {
                        #[link_name = concat!("cxxbridge1$seastar$shared_ptr$", $segment, "$get")]
                        fn __get(this: *const c_void) -> *const c_void;
                    }
                }
                unsafe { __get(this) }.cast()
            }
            unsafe fn __drop(this: *mut c_void) {
                extern "C" {
                    attr! {
                        #[link_name = concat!("cxxbridge1$seastar$shared_ptr$", $segment, "$drop")]
                        fn __drop(this: *mut c_void);
                    }
                }
                unsafe { __drop(this) }
            }
        }
    };
}

macro_rules! impl_shared_ptr_target_for_primitive {
    ($ty:ident) => {
        impl_shared_ptr_target!(stringify!($ty), stringify!($ty), $ty);
    };
}

impl_shared_ptr_target_for_primitive!(bool);
impl_shared_ptr_target_for_primitive!(u8);
impl_shared_ptr_target_for_primitive!(u16);
impl_shared_ptr_target_for_primitive!(u32);
impl_shared_ptr_target_for_primitive!(u64);
impl_shared_ptr_target_for_primitive!(usize);
impl_shared_ptr_target_for_primitive!(i8);
impl_shared_ptr_target_for_primitive!(i16);
impl_shared_ptr_target_for_primitive!(i32);
impl_shared_ptr_target_for_primitive!(i64);
impl_shared_ptr_target_for_primitive!(isize);
impl_shared_ptr_target_for_primitive!(f32);
impl_shared_ptr_target_for_primitive!(f64);

impl_shared_ptr_target!("string", "CxxString", CxxString);
