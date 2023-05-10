use crate::fmt::display;
use crate::kind::Trivial;
use crate::string::CxxString;
use crate::ExternType;
use core::ffi::c_void;
use core::fmt::{self, Debug, Display};
use core::marker::PhantomData;
use core::mem::MaybeUninit;
use core::ops::Deref;

/// Binding to C++ `seastar::lw_shared_ptr<T>`.
#[repr(C)]
pub struct SeastarLwSharedPtr<T>
where
    T: SeastarLwSharedPtrTarget,
{
    repr: MaybeUninit<*mut c_void>,
    ty: PhantomData<T>,
}

impl<T> SeastarLwSharedPtr<T>
where
    T: SeastarLwSharedPtrTarget,
{
    /// Makes a new SeastarLwSharedPtr wrapping a null pointer.
    ///
    /// Matches the behavior of default-constructing a seastar::lw_shared\_ptr.
    pub fn null() -> Self {
        let mut lw_shared_ptr = MaybeUninit::<SeastarLwSharedPtr<T>>::uninit();
        let new = lw_shared_ptr.as_mut_ptr().cast();
        unsafe {
            T::__null(new);
            lw_shared_ptr.assume_init()
        }
    }

    /// Allocates memory on the heap and makes a SeastarLwSharedPtr owner for it.
    pub fn new(value: T) -> Self
    where
        T: ExternType<Kind = Trivial>,
    {
        let mut lw_shared_ptr = MaybeUninit::<SeastarLwSharedPtr<T>>::uninit();
        let new = lw_shared_ptr.as_mut_ptr().cast();
        unsafe {
            T::__new(value, new);
            lw_shared_ptr.assume_init()
        }
    }

    /// Checks whether the SeastarLwSharedPtr does not own an object.
    ///
    /// This is the opposite of [seastar::lw_shared_ptr\<T\>::operator bool].
    pub fn is_null(&self) -> bool {
        let this = self as *const Self as *const c_void;
        let ptr = unsafe { T::__get(this) };
        ptr.is_null()
    }

    /// Returns a reference to the object owned by this SeastarLwSharedPtr if any,
    /// otherwise None.
    pub fn as_ref(&self) -> Option<&T> {
        let this = self as *const Self as *const c_void;
        unsafe { T::__get(this).as_ref() }
    }
}

impl<T> Clone for SeastarLwSharedPtr<T>
where
    T: SeastarLwSharedPtrTarget,
{
    fn clone(&self) -> Self {
        let mut lw_shared_ptr = MaybeUninit::<SeastarLwSharedPtr<T>>::uninit();
        let new = lw_shared_ptr.as_mut_ptr().cast();
        let this = self as *const Self as *mut c_void;
        unsafe {
            T::__clone(this, new);
            lw_shared_ptr.assume_init()
        }
    }
}

// SeastarLwSharedPtr is not a self-referential type and is safe to move out of a Pin,
// regardless whether the pointer's target is Unpin.
impl<T> Unpin for SeastarLwSharedPtr<T> where T: SeastarLwSharedPtrTarget {}

impl<T> Drop for SeastarLwSharedPtr<T>
where
    T: SeastarLwSharedPtrTarget,
{
    fn drop(&mut self) {
        let this = self as *mut Self as *mut c_void;
        unsafe { T::__drop(this) }
    }
}

impl<T> Deref for SeastarLwSharedPtr<T>
where
    T: SeastarLwSharedPtrTarget,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match self.as_ref() {
            Some(target) => target,
            None => panic!(
                "called deref on a null SeastarLwSharedPtr<{}>",
                display(T::__typename),
            ),
        }
    }
}

impl<T> Debug for SeastarLwSharedPtr<T>
where
    T: Debug + SeastarLwSharedPtrTarget,
{
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self.as_ref() {
            None => formatter.write_str("nullptr"),
            Some(value) => Debug::fmt(value, formatter),
        }
    }
}

impl<T> Display for SeastarLwSharedPtr<T>
where
    T: Display + SeastarLwSharedPtrTarget,
{
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self.as_ref() {
            None => formatter.write_str("nullptr"),
            Some(value) => Display::fmt(value, formatter),
        }
    }
}

/// Trait bound for types which may be used as the `T` inside of a
/// `SeastarLwSharedPtr<T>` in generic code.
///
/// This trait has no publicly callable or implementable methods. Implementing
/// it outside of the CXX codebase is not supported.
///
/// # Example
///
/// A bound `T: SeastarLwSharedPtrTarget` may be necessary when manipulating
/// [`SeastarLwSharedPtr`] in generic code.
///
/// ```
/// use cxx::memory::{SeastarLwSharedPtr, SeastarLwSharedPtrTarget};
/// use std::fmt::Display;
///
/// pub fn take_generic_ptr<T>(ptr: SeastarLwSharedPtr<T>)
/// where
///     T: SeastarLwSharedPtrTarget + Display,
/// {
///     println!("the lw_shared_ptr points to: {}", *ptr);
/// }
/// ```
///
/// Writing the same generic function without a `SeastarLwSharedPtrTarget` trait bound
/// would not compile.
pub unsafe trait SeastarLwSharedPtrTarget {
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

macro_rules! impl_lw_shared_ptr_target {
    ($segment:expr, $name:expr, $ty:ty) => {
        unsafe impl SeastarLwSharedPtrTarget for $ty {
            fn __typename(f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str($name)
            }
            unsafe fn __null(new: *mut c_void) {
                extern "C" {
                    attr! {
                        #[link_name = concat!("cxxbridge1$seastar$lw_shared_ptr$", $segment, "$null")]
                        fn __null(new: *mut c_void);
                    }
                }
                unsafe { __null(new) }
            }
            unsafe fn __new(value: Self, new: *mut c_void) {
                extern "C" {
                    attr! {
                        #[link_name = concat!("cxxbridge1$seastar$lw_shared_ptr$", $segment, "$uninit")]
                        fn __uninit(new: *mut c_void) -> *mut c_void;
                    }
                }
                unsafe { __uninit(new).cast::<$ty>().write(value) }
            }
            unsafe fn __clone(this: *const c_void, new: *mut c_void) {
                extern "C" {
                    attr! {
                        #[link_name = concat!("cxxbridge1$seastar$lw_shared_ptr$", $segment, "$clone")]
                        fn __clone(this: *const c_void, new: *mut c_void);
                    }
                }
                unsafe { __clone(this, new) }
            }
            unsafe fn __get(this: *const c_void) -> *const Self {
                extern "C" {
                    attr! {
                        #[link_name = concat!("cxxbridge1$seastar$lw_shared_ptr$", $segment, "$get")]
                        fn __get(this: *const c_void) -> *const c_void;
                    }
                }
                unsafe { __get(this) }.cast()
            }
            unsafe fn __drop(this: *mut c_void) {
                extern "C" {
                    attr! {
                        #[link_name = concat!("cxxbridge1$seastar$lw_shared_ptr$", $segment, "$drop")]
                        fn __drop(this: *mut c_void);
                    }
                }
                unsafe { __drop(this) }
            }
        }
    };
}

macro_rules! impl_lw_shared_ptr_target_for_primitive {
    ($ty:ident) => {
        impl_lw_shared_ptr_target!(stringify!($ty), stringify!($ty), $ty);
    };
}

impl_lw_shared_ptr_target_for_primitive!(bool);
impl_lw_shared_ptr_target_for_primitive!(u8);
impl_lw_shared_ptr_target_for_primitive!(u16);
impl_lw_shared_ptr_target_for_primitive!(u32);
impl_lw_shared_ptr_target_for_primitive!(u64);
impl_lw_shared_ptr_target_for_primitive!(usize);
impl_lw_shared_ptr_target_for_primitive!(i8);
impl_lw_shared_ptr_target_for_primitive!(i16);
impl_lw_shared_ptr_target_for_primitive!(i32);
impl_lw_shared_ptr_target_for_primitive!(i64);
impl_lw_shared_ptr_target_for_primitive!(isize);
impl_lw_shared_ptr_target_for_primitive!(f32);
impl_lw_shared_ptr_target_for_primitive!(f64);

impl_lw_shared_ptr_target!("string", "CxxString", CxxString);
