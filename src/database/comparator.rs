//! All keys in leveldb are compared by their binary value unless
//! defined otherwise.
//!
//! Comparators allow to override this comparison.
//! The ordering of keys introduced by the comparator influences iteration order.
//! Databases written with one Comparator cannot be opened with another.
use cruzbit_leveldb_sys::*;
use libc::{c_char, c_void, size_t};
use std::cmp::Ordering;
use std::slice;

/// A comparator has two important functions:
///
/// * the name function returns a fixed name to detect errors when
///   opening databases with a different name
/// * The comparison implementation
pub trait Comparator {
    /// Return the name of the Comparator
    fn name(&self) -> *const c_char;
    /// compare two keys. This must implement a total ordering.
    fn compare(&self, a: &[u8], b: &[u8]) -> Ordering {
        a.cmp(b)
    }

    /// whether the comparator is the `DefaultComparator`
    fn null() -> bool {
        false
    }
}

/// DefaultComparator is the a stand in for "no comparator set"
#[derive(Copy, Clone)]
pub struct DefaultComparator;

/// # Safety
unsafe trait InternalComparator: Comparator
where
    Self: Sized,
{
    extern "C" fn name(state: *mut c_void) -> *const c_char {
        let x = unsafe { &*(state as *mut Self) };
        x.name()
    }

    extern "C" fn compare(
        state: *mut c_void,
        a: *const c_char,
        a_len: size_t,
        b: *const c_char,
        b_len: size_t,
    ) -> i32 {
        unsafe {
            let a_slice = slice::from_raw_parts::<u8>(a as *const u8, a_len);
            let b_slice = slice::from_raw_parts::<u8>(b as *const u8, b_len);
            let x = &*(state as *mut Self);

            match x.compare(a_slice, b_slice) {
                Ordering::Less => -1,
                Ordering::Equal => 0,
                Ordering::Greater => 1,
            }
        }
    }

    extern "C" fn destructor(state: *mut c_void) {
        let _x: Box<Self> = unsafe { Box::from_raw(state as *mut Self) };
        // let the Box fall out of scope and run the T's destructor
    }
}

unsafe impl<C: Comparator> InternalComparator for C {}

#[allow(missing_docs)]
pub fn create_comparator<T: Comparator>(x: Box<T>) -> *mut leveldb_comparator_t {
    unsafe {
        leveldb_comparator_create(
            Box::into_raw(x) as *mut c_void,
            <T as InternalComparator>::destructor,
            <T as InternalComparator>::compare,
            <T as InternalComparator>::name,
        )
    }
}

impl Comparator for DefaultComparator {
    fn name(&self) -> *const c_char {
        "default_comparator".as_ptr() as *const c_char
    }

    fn null() -> bool {
        true
    }
}
