

#![stable(feature = "rust1", since = "1.0.0")]

use std::any::Any;
use std::borrow;
use std::cell::Cell;
use std::cmp::Ordering;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::intrinsics::abort;
use std::marker::{self, Unpin, Unsize, PhantomData};
use std::mem::{self, align_of_val, forget, size_of_val};
use std::ops::{Deref, Receiver, CoerceUnsized, DispatchFromDyn};
use std::pin::Pin;
use std::convert::From;

#[stable(feature = "rust1", since = "1.0.0")]
pub struct Share<T: ?Sized> (Rc<T>);

#[stable(feature = "rust1", since = "1.0.0")]
impl<T: ?Sized> !marker::Send for Share<T> {}
#[stable(feature = "rust1", since = "1.0.0")]
impl<T: ?Sized> !marker::Sync for Share<T> {}

#[unstable(feature = "coerce_unsized", issue = "27732")]
impl<T: ?Sized + Unsize<U>, U: ?Sized> CoerceUnsized<Rc<U>> for Share<T> {}

#[unstable(feature = "dispatch_from_dyn", issue = "0")]
impl<T: ?Sized + Unsize<U>, U: ?Sized> DispatchFromDyn<Rc<U>> for Share<T> {}

impl<T> Share<T> {

    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn new(value: T) -> Share<T> {
        Share(Rc::new(value))
    }

    /// Constructs a new `Pin<Share<T>>`. If `T` does not implement `Unpin`, then
    /// `value` will be pinned in memory and unable to be moved.
    #[stable(feature = "pin", since = "1.33.0")]
    pub fn pin(value: T) -> Pin<Share<T>> {
        unsafe { Pin::new_unchecked(Rc::new(value)) }
    }

    /// Returns the contained value, if the `Rc` has exactly one strong reference.
    ///
    /// Otherwise, an [`Err`][result] is returned with the same `Rc` that was
    /// passed in.
    ///
    /// This will succeed even if there are outstanding weak references.
    ///
    /// [result]: ../../std/result/enum.Result.html
    ///
    /// # Examples
    ///
    /// ```
    /// use std::rc::Rc;
    ///
    /// let x = Rc::new(3);
    /// assert_eq!(Rc::try_unwrap(x), Ok(3));
    ///
    /// let x = Rc::new(4);
    /// let _y = Rc::clone(&x);
    /// assert_eq!(*Rc::try_unwrap(x).unwrap_err(), 4);
    /// ```
    #[inline]
    #[stable(feature = "rc_unique", since = "1.4.0")]
    pub fn try_unwrap(this: Self) -> Result<T, Self> {
        if Rc::strong_count(&this) == 1 {
            unsafe {
                let val = ptr::read(&*this); // copy the contained object

                // Indicate to Weaks that they can't be promoted by decrementing
                // the strong count, and then remove the implicit "strong weak"
                // pointer while also handling drop logic by just crafting a
                // fake Weak.
                this.dec_strong();
                let _weak = Weak { ptr: this.ptr };
                forget(this);
                Ok(val)
            }
        } else {
            Err(this)
        }
    }
}

impl<T: ?Sized> Share<T> {
    /// Consumes the `Rc`, returning the wrapped pointer.
    ///
    /// To avoid a memory leak the pointer must be converted back to an `Rc` using
    /// [`Rc::from_raw`][from_raw].
    ///
    /// [from_raw]: struct.Rc.html#method.from_raw
    ///
    /// # Examples
    ///
    /// ```
    /// use std::rc::Rc;
    ///
    /// let x = Rc::new(10);
    /// let x_ptr = Rc::into_raw(x);
    /// assert_eq!(unsafe { *x_ptr }, 10);
    /// ```
    #[stable(feature = "rc_raw", since = "1.17.0")]
    pub fn into_raw(this: Self) -> *const T {
        let ptr: *const T = &*this;
        mem::forget(this);
        ptr
    }

    /// Constructs an `Rc` from a raw pointer.
    ///
    /// The raw pointer must have been previously returned by a call to a
    /// [`Rc::into_raw`][into_raw].
    ///
    /// This function is unsafe because improper use may lead to memory problems. For example, a
    /// double-free may occur if the function is called twice on the same raw pointer.
    ///
    /// [into_raw]: struct.Rc.html#method.into_raw
    ///
    /// # Examples
    ///
    /// ```
    /// use std::rc::Rc;
    ///
    /// let x = Rc::new(10);
    /// let x_ptr = Rc::into_raw(x);
    ///
    /// unsafe {
    ///     // Convert back to an `Rc` to prevent leak.
    ///     let x = Rc::from_raw(x_ptr);
    ///     assert_eq!(*x, 10);
    ///
    ///     // Further calls to `Rc::from_raw(x_ptr)` would be memory unsafe.
    /// }
    ///
    /// // The memory was freed when `x` went out of scope above, so `x_ptr` is now dangling!
    /// ```
    #[stable(feature = "rc_raw", since = "1.17.0")]
    pub unsafe fn from_raw(ptr: *const T) -> Self {
        // Align the unsized value to the end of the RcBox.
        // Because it is ?Sized, it will always be the last field in memory.
        let align = align_of_val(&*ptr);
        let layout = Layout::new::<RcBox<()>>();
        let offset = (layout.size() + layout.padding_needed_for(align)) as isize;

        // Reverse the offset to find the original RcBox.
        let fake_ptr = ptr as *mut RcBox<T>;
        let rc_ptr = set_data_ptr(fake_ptr, (ptr as *mut u8).offset(-offset));

        Rc {
            ptr: NonNull::new_unchecked(rc_ptr),
            phantom: PhantomData,
        }
    }

    /// Consumes the `Rc`, returning the wrapped pointer as `NonNull<T>`.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(rc_into_raw_non_null)]
    ///
    /// use std::rc::Rc;
    ///
    /// let x = Rc::new(10);
    /// let ptr = Rc::into_raw_non_null(x);
    /// let deref = unsafe { *ptr.as_ref() };
    /// assert_eq!(deref, 10);
    /// ```
    #[unstable(feature = "rc_into_raw_non_null", issue = "47336")]
    #[inline]
    pub fn into_raw_non_null(this: Self) -> NonNull<T> {
        // safe because Rc guarantees its pointer is non-null
        unsafe { NonNull::new_unchecked(Rc::into_raw(this) as *mut _) }
    }

    /// Creates a new [`Weak`][weak] pointer to this value.
    ///
    /// [weak]: struct.Weak.html
    ///
    /// # Examples
    ///
    /// ```
    /// use std::rc::Rc;
    ///
    /// let five = Rc::new(5);
    ///
    /// let weak_five = Rc::downgrade(&five);
    /// ```
    #[stable(feature = "rc_weak", since = "1.4.0")]
    pub fn downgrade(this: &Self) -> Weak<T> {
        this.inc_weak();
        // Make sure we do not create a dangling Weak
        debug_assert!(!is_dangling(this.ptr));
        Weak { ptr: this.ptr }
    }

    /// Gets the number of [`Weak`][weak] pointers to this value.
    ///
    /// [weak]: struct.Weak.html
    ///
    /// # Examples
    ///
    /// ```
    /// use std::rc::Rc;
    ///
    /// let five = Rc::new(5);
    /// let _weak_five = Rc::downgrade(&five);
    ///
    /// assert_eq!(1, Rc::weak_count(&five));
    /// ```
    #[inline]
    #[stable(feature = "rc_counts", since = "1.15.0")]
    pub fn weak_count(this: &Self) -> usize {
        this.weak() - 1
    }

    /// Gets the number of strong (`Rc`) pointers to this value.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::rc::Rc;
    ///
    /// let five = Rc::new(5);
    /// let _also_five = Rc::clone(&five);
    ///
    /// assert_eq!(2, Rc::strong_count(&five));
    /// ```
    #[inline]
    #[stable(feature = "rc_counts", since = "1.15.0")]
    pub fn strong_count(this: &Self) -> usize {
        this.strong()
    }

    /// Returns `true` if there are no other `Rc` or [`Weak`][weak] pointers to
    /// this inner value.
    ///
    /// [weak]: struct.Weak.html
    #[inline]
    fn is_unique(this: &Self) -> bool {
        Rc::weak_count(this) == 0 && Rc::strong_count(this) == 1
    }

    /// Returns a mutable reference to the inner value, if there are
    /// no other `Rc` or [`Weak`][weak] pointers to the same value.
    ///
    /// Returns [`None`] otherwise, because it is not safe to
    /// mutate a shared value.
    ///
    /// See also [`make_mut`][make_mut], which will [`clone`][clone]
    /// the inner value when it's shared.
    ///
    /// [weak]: struct.Weak.html
    /// [`None`]: ../../std/option/enum.Option.html#variant.None
    /// [make_mut]: struct.Rc.html#method.make_mut
    /// [clone]: ../../std/clone/trait.Clone.html#tymethod.clone
    ///
    /// # Examples
    ///
    /// ```
    /// use std::rc::Rc;
    ///
    /// let mut x = Rc::new(3);
    /// *Rc::get_mut(&mut x).unwrap() = 4;
    /// assert_eq!(*x, 4);
    ///
    /// let _y = Rc::clone(&x);
    /// assert!(Rc::get_mut(&mut x).is_none());
    /// ```
    #[inline]
    #[stable(feature = "rc_unique", since = "1.4.0")]
    pub fn get_mut(this: &mut Self) -> Option<&mut T> {
        if Rc::is_unique(this) {
            unsafe {
                Some(&mut this.ptr.as_mut().value)
            }
        } else {
            None
        }
    }

    #[inline]
    #[stable(feature = "ptr_eq", since = "1.17.0")]
    /// Returns `true` if the two `Rc`s point to the same value (not
    /// just values that compare as equal).
    ///
    /// # Examples
    ///
    /// ```
    /// use std::rc::Rc;
    ///
    /// let five = Rc::new(5);
    /// let same_five = Rc::clone(&five);
    /// let other_five = Rc::new(5);
    ///
    /// assert!(Rc::ptr_eq(&five, &same_five));
    /// assert!(!Rc::ptr_eq(&five, &other_five));
    /// ```
    pub fn ptr_eq(this: &Self, other: &Self) -> bool {
        this.ptr.as_ptr() == other.ptr.as_ptr()
    }
}

impl<T: Clone> Share<T> {
    /// Makes a mutable reference into the given `Rc`.
    ///
    /// If there are other `Rc` or [`Weak`][weak] pointers to the same value,
    /// then `make_mut` will invoke [`clone`][clone] on the inner value to
    /// ensure unique ownership. This is also referred to as clone-on-write.
    ///
    /// See also [`get_mut`][get_mut], which will fail rather than cloning.
    ///
    /// [weak]: struct.Weak.html
    /// [clone]: ../../std/clone/trait.Clone.html#tymethod.clone
    /// [get_mut]: struct.Rc.html#method.get_mut
    ///
    /// # Examples
    ///
    /// ```
    /// use std::rc::Rc;
    ///
    /// let mut data = Rc::new(5);
    ///
    /// *Rc::make_mut(&mut data) += 1;        // Won't clone anything
    /// let mut other_data = Rc::clone(&data);    // Won't clone inner data
    /// *Rc::make_mut(&mut data) += 1;        // Clones inner data
    /// *Rc::make_mut(&mut data) += 1;        // Won't clone anything
    /// *Rc::make_mut(&mut other_data) *= 2;  // Won't clone anything
    ///
    /// // Now `data` and `other_data` point to different values.
    /// assert_eq!(*data, 8);
    /// assert_eq!(*other_data, 12);
    /// ```
    #[inline]
    #[stable(feature = "rc_unique", since = "1.4.0")]
    pub fn make_mut(this: &mut Self) -> &mut T {
        if Rc::strong_count(this) != 1 {
            // Gotta clone the data, there are other Rcs
            *this = Rc::new((**this).clone())
        } else if Rc::weak_count(this) != 0 {
            // Can just steal the data, all that's left is Weaks
            unsafe {
                let mut swap = Rc::new(ptr::read(&this.ptr.as_ref().value));
                mem::swap(this, &mut swap);
                swap.dec_strong();
                // Remove implicit strong-weak ref (no need to craft a fake
                // Weak here -- we know other Weaks can clean up for us)
                swap.dec_weak();
                forget(swap);
            }
        }
        // This unsafety is ok because we're guaranteed that the pointer
        // returned is the *only* pointer that will ever be returned to T. Our
        // reference count is guaranteed to be 1 at this point, and we required
        // the `Share<T>` itself to be `mut`, so we're returning the only possible
        // reference to the inner value.
        unsafe {
            &mut this.ptr.as_mut().value
        }
    }
}

impl Rc<dyn Any> {
    #[inline]
    #[stable(feature = "rc_downcast", since = "1.29.0")]
    /// Attempt to downcast the `Rc<dyn Any>` to a concrete type.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::any::Any;
    /// use std::rc::Rc;
    ///
    /// fn print_if_string(value: Rc<dyn Any>) {
    ///     if let Ok(string) = value.downcast::<String>() {
    ///         println!("String ({}): {}", string.len(), string);
    ///     }
    /// }
    ///
    /// fn main() {
    ///     let my_string = "Hello World".to_string();
    ///     print_if_string(Rc::new(my_string));
    ///     print_if_string(Rc::new(0i8));
    /// }
    /// ```
    pub fn downcast<T: Any>(self) -> Result<Share<T>, Rc<dyn Any>> {
        if (*self).is::<T>() {
            let ptr = self.ptr.cast::<RcBox<T>>();
            forget(self);
            Ok(Rc { ptr, phantom: PhantomData })
        } else {
            Err(self)
        }
    }
}

impl<T: ?Sized> Share<T> {
    // Allocates an `RcBox<T>` with sufficient space for an unsized value
    unsafe fn allocate_for_ptr(ptr: *const T) -> *mut RcBox<T> {
        // Calculate layout using the given value.
        // Previously, layout was calculated on the expression
        // `&*(ptr as *const RcBox<T>)`, but this created a misaligned
        // reference (see #54908).
        let layout = Layout::new::<RcBox<()>>()
            .extend(Layout::for_value(&*ptr)).unwrap().0
            .pad_to_align().unwrap();

        let mem = Global.alloc(layout)
            .unwrap_or_else(|_| handle_alloc_error(layout));

        // Initialize the RcBox
        let inner = set_data_ptr(ptr as *mut T, mem.as_ptr() as *mut u8) as *mut RcBox<T>;
        debug_assert_eq!(Layout::for_value(&*inner), layout);

        ptr::write(&mut (*inner).strong, Cell::new(1));
        ptr::write(&mut (*inner).weak, Cell::new(1));

        inner
    }

    fn from_box(v: Box<T>) -> Share<T> {
        unsafe {
            let box_unique = Box::into_unique(v);
            let bptr = box_unique.as_ptr();

            let value_size = size_of_val(&*bptr);
            let ptr = Self::allocate_for_ptr(bptr);

            // Copy value as bytes
            ptr::copy_nonoverlapping(
                bptr as *const T as *const u8,
                &mut (*ptr).value as *mut _ as *mut u8,
                value_size);

            // Free the allocation without dropping its contents
            box_free(box_unique);

            Rc { ptr: NonNull::new_unchecked(ptr), phantom: PhantomData }
        }
    }
}

// Sets the data pointer of a `?Sized` raw pointer.
//
// For a slice/trait object, this sets the `data` field and leaves the rest
// unchanged. For a sized raw pointer, this simply sets the pointer.
unsafe fn set_data_ptr<T: ?Sized, U>(mut ptr: *mut T, data: *mut U) -> *mut T {
    ptr::write(&mut ptr as *mut _ as *mut *mut u8, data as *mut u8);
    ptr
}

impl<T> Rc<[T]> {
    // Copy elements from slice into newly allocated Rc<[T]>
    //
    // Unsafe because the caller must either take ownership or bind `T: Copy`
    unsafe fn copy_from_slice(v: &[T]) -> Rc<[T]> {
        let v_ptr = v as *const [T];
        let ptr = Self::allocate_for_ptr(v_ptr);

        ptr::copy_nonoverlapping(
            v.as_ptr(),
            &mut (*ptr).value as *mut [T] as *mut T,
            v.len());

        Rc { ptr: NonNull::new_unchecked(ptr), phantom: PhantomData }
    }
}

trait RcFromSlice<T> {
    fn from_slice(slice: &[T]) -> Self;
}

impl<T: Clone> RcFromSlice<T> for Rc<[T]> {
    #[inline]
    default fn from_slice(v: &[T]) -> Self {
        // Panic guard while cloning T elements.
        // In the event of a panic, elements that have been written
        // into the new RcBox will be dropped, then the memory freed.
        struct Guard<T> {
            mem: NonNull<u8>,
            elems: *mut T,
            layout: Layout,
            n_elems: usize,
        }

        impl<T> Drop for Guard<T> {
            fn drop(&mut self) {
                unsafe {
                    let slice = from_raw_parts_mut(self.elems, self.n_elems);
                    ptr::drop_in_place(slice);

                    Global.dealloc(self.mem, self.layout.clone());
                }
            }
        }

        unsafe {
            let v_ptr = v as *const [T];
            let ptr = Self::allocate_for_ptr(v_ptr);

            let mem = ptr as *mut _ as *mut u8;
            let layout = Layout::for_value(&*ptr);

            // Pointer to first element
            let elems = &mut (*ptr).value as *mut [T] as *mut T;

            let mut guard = Guard{
                mem: NonNull::new_unchecked(mem),
                elems: elems,
                layout: layout,
                n_elems: 0,
            };

            for (i, item) in v.iter().enumerate() {
                ptr::write(elems.add(i), item.clone());
                guard.n_elems += 1;
            }

            // All clear. Forget the guard so it doesn't free the new RcBox.
            forget(guard);

            Rc { ptr: NonNull::new_unchecked(ptr), phantom: PhantomData }
        }
    }
}

impl<T: Copy> RcFromSlice<T> for Rc<[T]> {
    #[inline]
    fn from_slice(v: &[T]) -> Self {
        unsafe { Rc::copy_from_slice(v) }
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<T: ?Sized> Deref for Share<T> {
    type Target = T;

    #[inline(always)]
    fn deref(&self) -> &T {
        &self.inner().value
    }
}

#[unstable(feature = "receiver_trait", issue = "0")]
impl<T: ?Sized> Receiver for Share<T> {}

#[stable(feature = "rust1", since = "1.0.0")]
unsafe impl<#[may_dangle] T: ?Sized> Drop for Share<T> {
    /// Drops the `Rc`.
    ///
    /// This will decrement the strong reference count. If the strong reference
    /// count reaches zero then the only other references (if any) are
    /// [`Weak`], so we `drop` the inner value.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::rc::Rc;
    ///
    /// struct Foo;
    ///
    /// impl Drop for Foo {
    ///     fn drop(&mut self) {
    ///         println!("dropped!");
    ///     }
    /// }
    ///
    /// let foo  = Rc::new(Foo);
    /// let foo2 = Rc::clone(&foo);
    ///
    /// drop(foo);    // Doesn't print anything
    /// drop(foo2);   // Prints "dropped!"
    /// ```
    ///
    /// [`Weak`]: ../../std/rc/struct.Weak.html
    fn drop(&mut self) {
        unsafe {
            self.dec_strong();
            if self.strong() == 0 {
                // destroy the contained object
                ptr::drop_in_place(self.ptr.as_mut());

                // remove the implicit "strong weak" pointer now that we've
                // destroyed the contents.
                self.dec_weak();

                if self.weak() == 0 {
                    Global.dealloc(self.ptr.cast(), Layout::for_value(self.ptr.as_ref()));
                }
            }
        }
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<T: ?Sized> Clone for Share<T> {
    /// Makes a clone of the `Rc` pointer.
    ///
    /// This creates another pointer to the same inner value, increasing the
    /// strong reference count.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::rc::Rc;
    ///
    /// let five = Rc::new(5);
    ///
    /// let _ = Rc::clone(&five);
    /// ```
    #[inline]
    fn clone(&self) -> Share<T> {
        self.inc_strong();
        Rc { ptr: self.ptr, phantom: PhantomData }
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<T: Default> Default for Share<T> {
    /// Creates a new `Share<T>`, with the `Default` value for `T`.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::rc::Rc;
    ///
    /// let x: Rc<i32> = Default::default();
    /// assert_eq!(*x, 0);
    /// ```
    #[inline]
    fn default() -> Share<T> {
        Rc::new(Default::default())
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
trait RcEqIdent<T: ?Sized + PartialEq> {
    fn eq(&self, other: &Share<T>) -> bool;
    fn ne(&self, other: &Share<T>) -> bool;
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<T: ?Sized + PartialEq> RcEqIdent<T> for Share<T> {
    #[inline]
    default fn eq(&self, other: &Share<T>) -> bool {
        **self == **other
    }

    #[inline]
    default fn ne(&self, other: &Share<T>) -> bool {
        **self != **other
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<T: ?Sized + Eq> RcEqIdent<T> for Share<T> {
    #[inline]
    fn eq(&self, other: &Share<T>) -> bool {
        Rc::ptr_eq(self, other) || **self == **other
    }

    #[inline]
    fn ne(&self, other: &Share<T>) -> bool {
        !Rc::ptr_eq(self, other) && **self != **other
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<T: ?Sized + PartialEq> PartialEq for Share<T> {
    /// Equality for two `Rc`s.
    ///
    /// Two `Rc`s are equal if their inner values are equal.
    ///
    /// If `T` also implements `Eq`, two `Rc`s that point to the same value are
    /// always equal.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::rc::Rc;
    ///
    /// let five = Rc::new(5);
    ///
    /// assert!(five == Rc::new(5));
    /// ```
    #[inline]
    fn eq(&self, other: &Share<T>) -> bool {
        RcEqIdent::eq(self, other)
    }

    /// Inequality for two `Rc`s.
    ///
    /// Two `Rc`s are unequal if their inner values are unequal.
    ///
    /// If `T` also implements `Eq`, two `Rc`s that point to the same value are
    /// never unequal.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::rc::Rc;
    ///
    /// let five = Rc::new(5);
    ///
    /// assert!(five != Rc::new(6));
    /// ```
    #[inline]
    fn ne(&self, other: &Share<T>) -> bool {
        RcEqIdent::ne(self, other)
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<T: ?Sized + Eq> Eq for Share<T> {}

#[stable(feature = "rust1", since = "1.0.0")]
impl<T: ?Sized + PartialOrd> PartialOrd for Share<T> {
    /// Partial comparison for two `Rc`s.
    ///
    /// The two are compared by calling `partial_cmp()` on their inner values.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::rc::Rc;
    /// use std::cmp::Ordering;
    ///
    /// let five = Rc::new(5);
    ///
    /// assert_eq!(Some(Ordering::Less), five.partial_cmp(&Rc::new(6)));
    /// ```
    #[inline(always)]
    fn partial_cmp(&self, other: &Share<T>) -> Option<Ordering> {
        (**self).partial_cmp(&**other)
    }

    /// Less-than comparison for two `Rc`s.
    ///
    /// The two are compared by calling `<` on their inner values.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::rc::Rc;
    ///
    /// let five = Rc::new(5);
    ///
    /// assert!(five < Rc::new(6));
    /// ```
    #[inline(always)]
    fn lt(&self, other: &Share<T>) -> bool {
        **self < **other
    }

    /// 'Less than or equal to' comparison for two `Rc`s.
    ///
    /// The two are compared by calling `<=` on their inner values.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::rc::Rc;
    ///
    /// let five = Rc::new(5);
    ///
    /// assert!(five <= Rc::new(5));
    /// ```
    #[inline(always)]
    fn le(&self, other: &Share<T>) -> bool {
        **self <= **other
    }

    /// Greater-than comparison for two `Rc`s.
    ///
    /// The two are compared by calling `>` on their inner values.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::rc::Rc;
    ///
    /// let five = Rc::new(5);
    ///
    /// assert!(five > Rc::new(4));
    /// ```
    #[inline(always)]
    fn gt(&self, other: &Share<T>) -> bool {
        **self > **other
    }

    /// 'Greater than or equal to' comparison for two `Rc`s.
    ///
    /// The two are compared by calling `>=` on their inner values.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::rc::Rc;
    ///
    /// let five = Rc::new(5);
    ///
    /// assert!(five >= Rc::new(5));
    /// ```
    #[inline(always)]
    fn ge(&self, other: &Share<T>) -> bool {
        **self >= **other
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<T: ?Sized + Ord> Ord for Share<T> {
    /// Comparison for two `Rc`s.
    ///
    /// The two are compared by calling `cmp()` on their inner values.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::rc::Rc;
    /// use std::cmp::Ordering;
    ///
    /// let five = Rc::new(5);
    ///
    /// assert_eq!(Ordering::Less, five.cmp(&Rc::new(6)));
    /// ```
    #[inline]
    fn cmp(&self, other: &Share<T>) -> Ordering {
        (**self).cmp(&**other)
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<T: ?Sized + Hash> Hash for Share<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        (**self).hash(state);
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<T: ?Sized + fmt::Display> fmt::Display for Share<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&**self, f)
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<T: ?Sized + fmt::Debug> fmt::Debug for Share<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<T: ?Sized> fmt::Pointer for Share<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Pointer::fmt(&(&**self as *const T), f)
    }
}

#[stable(feature = "from_for_ptrs", since = "1.6.0")]
impl<T> From<T> for Share<T> {
    fn from(t: T) -> Self {
        Rc::new(t)
    }
}

#[stable(feature = "shared_from_slice", since = "1.21.0")]
impl<T: Clone> From<&[T]> for Rc<[T]> {
    #[inline]
    fn from(v: &[T]) -> Rc<[T]> {
        <Self as RcFromSlice<T>>::from_slice(v)
    }
}

#[stable(feature = "shared_from_slice", since = "1.21.0")]
impl From<&str> for Rc<str> {
    #[inline]
    fn from(v: &str) -> Rc<str> {
        let rc = Rc::<[u8]>::from(v.as_bytes());
        unsafe { Rc::from_raw(Rc::into_raw(rc) as *const str) }
    }
}

#[stable(feature = "shared_from_slice", since = "1.21.0")]
impl From<String> for Rc<str> {
    #[inline]
    fn from(v: String) -> Rc<str> {
        Rc::from(&v[..])
    }
}

#[stable(feature = "shared_from_slice", since = "1.21.0")]
impl<T: ?Sized> From<Box<T>> for Share<T> {
    #[inline]
    fn from(v: Box<T>) -> Share<T> {
        Rc::from_box(v)
    }
}

#[stable(feature = "shared_from_slice", since = "1.21.0")]
impl<T> From<Vec<T>> for Rc<[T]> {
    #[inline]
    fn from(mut v: Vec<T>) -> Rc<[T]> {
        unsafe {
            let rc = Rc::copy_from_slice(&v);

            // Allow the Vec to free its memory, but not destroy its contents
            v.set_len(0);

            rc
        }
    }
}

/// `Weak` is a version of [`Rc`] that holds a non-owning reference to the
/// managed value. The value is accessed by calling [`upgrade`] on the `Weak`
/// pointer, which returns an [`Option`]`<`[`Rc`]`<T>>`.
///
/// Since a `Weak` reference does not count towards ownership, it will not
/// prevent the inner value from being dropped, and `Weak` itself makes no
/// guarantees about the value still being present and may return [`None`]
/// when [`upgrade`]d.
///
/// A `Weak` pointer is useful for keeping a temporary reference to the value
/// within [`Rc`] without extending its lifetime. It is also used to prevent
/// circular references between [`Rc`] pointers, since mutual owning references
/// would never allow either [`Rc`] to be dropped. For example, a tree could
/// have strong [`Rc`] pointers from parent nodes to children, and `Weak`
/// pointers from children back to their parents.
///
/// The typical way to obtain a `Weak` pointer is to call [`Rc::downgrade`].
///
/// [`Rc`]: struct.Rc.html
/// [`Rc::downgrade`]: struct.Rc.html#method.downgrade
/// [`upgrade`]: struct.Weak.html#method.upgrade
/// [`Option`]: ../../std/option/enum.Option.html
/// [`None`]: ../../std/option/enum.Option.html#variant.None
#[stable(feature = "rc_weak", since = "1.4.0")]
pub struct Weak<T: ?Sized> {
    // This is a `NonNull` to allow optimizing the size of this type in enums,
    // but it is not necessarily a valid pointer.
    // `Weak::new` sets this to `usize::MAX` so that it doesn’t need
    // to allocate space on the heap.  That's not a value a real pointer
    // will ever have because RcBox has alignment at least 2.
    ptr: NonNull<RcBox<T>>,
}

#[stable(feature = "rc_weak", since = "1.4.0")]
impl<T: ?Sized> !marker::Send for Weak<T> {}
#[stable(feature = "rc_weak", since = "1.4.0")]
impl<T: ?Sized> !marker::Sync for Weak<T> {}

#[unstable(feature = "coerce_unsized", issue = "27732")]
impl<T: ?Sized + Unsize<U>, U: ?Sized> CoerceUnsized<Weak<U>> for Weak<T> {}

#[unstable(feature = "dispatch_from_dyn", issue = "0")]
impl<T: ?Sized + Unsize<U>, U: ?Sized> DispatchFromDyn<Weak<U>> for Weak<T> {}

impl<T> Weak<T> {
    /// Constructs a new `Weak<T>`, without allocating any memory.
    /// Calling [`upgrade`] on the return value always gives [`None`].
    ///
    /// [`upgrade`]: #method.upgrade
    /// [`None`]: ../../std/option/enum.Option.html
    ///
    /// # Examples
    ///
    /// ```
    /// use std::rc::Weak;
    ///
    /// let empty: Weak<i64> = Weak::new();
    /// assert!(empty.upgrade().is_none());
    /// ```
    #[stable(feature = "downgraded_weak", since = "1.10.0")]
    pub fn new() -> Weak<T> {
        Weak {
            ptr: NonNull::new(usize::MAX as *mut RcBox<T>).expect("MAX is not 0"),
        }
    }
}

pub(crate) fn is_dangling<T: ?Sized>(ptr: NonNull<T>) -> bool {
    let address = ptr.as_ptr() as *mut () as usize;
    address == usize::MAX
}

impl<T: ?Sized> Weak<T> {
    /// Attempts to upgrade the `Weak` pointer to an [`Rc`], extending
    /// the lifetime of the value if successful.
    ///
    /// Returns [`None`] if the value has since been dropped.
    ///
    /// [`Rc`]: struct.Rc.html
    /// [`None`]: ../../std/option/enum.Option.html
    ///
    /// # Examples
    ///
    /// ```
    /// use std::rc::Rc;
    ///
    /// let five = Rc::new(5);
    ///
    /// let weak_five = Rc::downgrade(&five);
    ///
    /// let strong_five: Option<Rc<_>> = weak_five.upgrade();
    /// assert!(strong_five.is_some());
    ///
    /// // Destroy all strong pointers.
    /// drop(strong_five);
    /// drop(five);
    ///
    /// assert!(weak_five.upgrade().is_none());
    /// ```
    #[stable(feature = "rc_weak", since = "1.4.0")]
    pub fn upgrade(&self) -> Option<Share<T>> {
        let inner = self.inner()?;
        if inner.strong() == 0 {
            None
        } else {
            inner.inc_strong();
            Some(Rc { ptr: self.ptr, phantom: PhantomData })
        }
    }

    /// Gets the number of strong (`Rc`) pointers pointing to this value.
    ///
    /// If `self` was created using [`Weak::new`], this will return 0.
    ///
    /// [`Weak::new`]: #method.new
    #[unstable(feature = "weak_counts", issue = "57977")]
    pub fn strong_count(&self) -> usize {
        if let Some(inner) = self.inner() {
            inner.strong()
        } else {
            0
        }
    }

    /// Gets the number of `Weak` pointers pointing to this value.
    ///
    /// If `self` was created using [`Weak::new`], this will return `None`. If
    /// not, the returned value is at least 1, since `self` still points to the
    /// value.
    ///
    /// [`Weak::new`]: #method.new
    #[unstable(feature = "weak_counts", issue = "57977")]
    pub fn weak_count(&self) -> Option<usize> {
        self.inner().map(|inner| {
            if inner.strong() > 0 {
                inner.weak() - 1  // subtract the implicit weak ptr
            } else {
                inner.weak()
            }
        })
    }

    /// Returns `None` when the pointer is dangling and there is no allocated `RcBox`
    /// (i.e., when this `Weak` was created by `Weak::new`).
    #[inline]
    fn inner(&self) -> Option<&RcBox<T>> {
        if is_dangling(self.ptr) {
            None
        } else {
            Some(unsafe { self.ptr.as_ref() })
        }
    }

    /// Returns `true` if the two `Weak`s point to the same value (not just values
    /// that compare as equal).
    ///
    /// # Notes
    ///
    /// Since this compares pointers it means that `Weak::new()` will equal each
    /// other, even though they don't point to any value.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(weak_ptr_eq)]
    /// use std::rc::{Rc, Weak};
    ///
    /// let first_rc = Rc::new(5);
    /// let first = Rc::downgrade(&first_rc);
    /// let second = Rc::downgrade(&first_rc);
    ///
    /// assert!(Weak::ptr_eq(&first, &second));
    ///
    /// let third_rc = Rc::new(5);
    /// let third = Rc::downgrade(&third_rc);
    ///
    /// assert!(!Weak::ptr_eq(&first, &third));
    /// ```
    ///
    /// Comparing `Weak::new`.
    ///
    /// ```
    /// #![feature(weak_ptr_eq)]
    /// use std::rc::{Rc, Weak};
    ///
    /// let first = Weak::new();
    /// let second = Weak::new();
    /// assert!(Weak::ptr_eq(&first, &second));
    ///
    /// let third_rc = Rc::new(());
    /// let third = Rc::downgrade(&third_rc);
    /// assert!(!Weak::ptr_eq(&first, &third));
    /// ```
    #[inline]
    #[unstable(feature = "weak_ptr_eq", issue = "55981")]
    pub fn ptr_eq(this: &Self, other: &Self) -> bool {
        this.ptr.as_ptr() == other.ptr.as_ptr()
    }
}

#[stable(feature = "rc_weak", since = "1.4.0")]
impl<T: ?Sized> Drop for Weak<T> {
    /// Drops the `Weak` pointer.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::rc::{Rc, Weak};
    ///
    /// struct Foo;
    ///
    /// impl Drop for Foo {
    ///     fn drop(&mut self) {
    ///         println!("dropped!");
    ///     }
    /// }
    ///
    /// let foo = Rc::new(Foo);
    /// let weak_foo = Rc::downgrade(&foo);
    /// let other_weak_foo = Weak::clone(&weak_foo);
    ///
    /// drop(weak_foo);   // Doesn't print anything
    /// drop(foo);        // Prints "dropped!"
    ///
    /// assert!(other_weak_foo.upgrade().is_none());
    /// ```
    fn drop(&mut self) {
        if let Some(inner) = self.inner() {
            inner.dec_weak();
            // the weak count starts at 1, and will only go to zero if all
            // the strong pointers have disappeared.
            if inner.weak() == 0 {
                unsafe {
                    Global.dealloc(self.ptr.cast(), Layout::for_value(self.ptr.as_ref()));
                }
            }
        }
    }
}

#[stable(feature = "rc_weak", since = "1.4.0")]
impl<T: ?Sized> Clone for Weak<T> {
    /// Makes a clone of the `Weak` pointer that points to the same value.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::rc::{Rc, Weak};
    ///
    /// let weak_five = Rc::downgrade(&Rc::new(5));
    ///
    /// let _ = Weak::clone(&weak_five);
    /// ```
    #[inline]
    fn clone(&self) -> Weak<T> {
        if let Some(inner) = self.inner() {
            inner.inc_weak()
        }
        Weak { ptr: self.ptr }
    }
}

#[stable(feature = "rc_weak", since = "1.4.0")]
impl<T: ?Sized + fmt::Debug> fmt::Debug for Weak<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(Weak)")
    }
}

#[stable(feature = "downgraded_weak", since = "1.10.0")]
impl<T> Default for Weak<T> {
    /// Constructs a new `Weak<T>`, allocating memory for `T` without initializing
    /// it. Calling [`upgrade`] on the return value always gives [`None`].
    ///
    /// [`None`]: ../../std/option/enum.Option.html
    /// [`upgrade`]: ../../std/rc/struct.Weak.html#method.upgrade
    ///
    /// # Examples
    ///
    /// ```
    /// use std::rc::Weak;
    ///
    /// let empty: Weak<i64> = Default::default();
    /// assert!(empty.upgrade().is_none());
    /// ```
    fn default() -> Weak<T> {
        Weak::new()
    }
}

// NOTE: We checked_add here to deal with mem::forget safely. In particular
// if you mem::forget Rcs (or Weaks), the ref-count can overflow, and then
// you can free the allocation while outstanding Rcs (or Weaks) exist.
// We abort because this is such a degenerate scenario that we don't care about
// what happens -- no real program should ever experience this.
//
// This should have negligible overhead since you don't actually need to
// clone these much in Rust thanks to ownership and move-semantics.

#[doc(hidden)]
trait RcBoxPtr<T: ?Sized> {
    fn inner(&self) -> &RcBox<T>;

    #[inline]
    fn strong(&self) -> usize {
        self.inner().strong.get()
    }

    #[inline]
    fn inc_strong(&self) {
        // We want to abort on overflow instead of dropping the value.
        // The reference count will never be zero when this is called;
        // nevertheless, we insert an abort here to hint LLVM at
        // an otherwise missed optimization.
        if self.strong() == 0 || self.strong() == usize::max_value() {
            unsafe { abort(); }
        }
        self.inner().strong.set(self.strong() + 1);
    }

    #[inline]
    fn dec_strong(&self) {
        self.inner().strong.set(self.strong() - 1);
    }

    #[inline]
    fn weak(&self) -> usize {
        self.inner().weak.get()
    }

    #[inline]
    fn inc_weak(&self) {
        // We want to abort on overflow instead of dropping the value.
        // The reference count will never be zero when this is called;
        // nevertheless, we insert an abort here to hint LLVM at
        // an otherwise missed optimization.
        if self.weak() == 0 || self.weak() == usize::max_value() {
            unsafe { abort(); }
        }
        self.inner().weak.set(self.weak() + 1);
    }

    #[inline]
    fn dec_weak(&self) {
        self.inner().weak.set(self.weak() - 1);
    }
}

impl<T: ?Sized> RcBoxPtr<T> for Share<T> {
    #[inline(always)]
    fn inner(&self) -> &RcBox<T> {
        unsafe {
            self.ptr.as_ref()
        }
    }
}

impl<T: ?Sized> RcBoxPtr<T> for RcBox<T> {
    #[inline(always)]
    fn inner(&self) -> &RcBox<T> {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::{Rc, Weak};
    use std::boxed::Box;
    use std::cell::RefCell;
    use std::option::Option::{self, None, Some};
    use std::result::Result::{Err, Ok};
    use std::mem::drop;
    use std::clone::Clone;
    use std::convert::From;

    #[test]
    fn test_clone() {
        let x = Rc::new(RefCell::new(5));
        let y = x.clone();
        *x.borrow_mut() = 20;
        assert_eq!(*y.borrow(), 20);
    }

    #[test]
    fn test_simple() {
        let x = Rc::new(5);
        assert_eq!(*x, 5);
    }

    #[test]
    fn test_simple_clone() {
        let x = Rc::new(5);
        let y = x.clone();
        assert_eq!(*x, 5);
        assert_eq!(*y, 5);
    }

    #[test]
    fn test_destructor() {
        let x: Rc<Box<_>> = Rc::new(box 5);
        assert_eq!(**x, 5);
    }

    #[test]
    fn test_live() {
        let x = Rc::new(5);
        let y = Rc::downgrade(&x);
        assert!(y.upgrade().is_some());
    }

    #[test]
    fn test_dead() {
        let x = Rc::new(5);
        let y = Rc::downgrade(&x);
        drop(x);
        assert!(y.upgrade().is_none());
    }

    #[test]
    fn weak_self_cyclic() {
        struct Cycle {
            x: RefCell<Option<Weak<Cycle>>>,
        }

        let a = Rc::new(Cycle { x: RefCell::new(None) });
        let b = Rc::downgrade(&a.clone());
        *a.x.borrow_mut() = Some(b);

        // hopefully we don't double-free (or leak)...
    }

    #[test]
    fn is_unique() {
        let x = Rc::new(3);
        assert!(Rc::is_unique(&x));
        let y = x.clone();
        assert!(!Rc::is_unique(&x));
        drop(y);
        assert!(Rc::is_unique(&x));
        let w = Rc::downgrade(&x);
        assert!(!Rc::is_unique(&x));
        drop(w);
        assert!(Rc::is_unique(&x));
    }

    #[test]
    fn test_strong_count() {
        let a = Rc::new(0);
        assert!(Rc::strong_count(&a) == 1);
        let w = Rc::downgrade(&a);
        assert!(Rc::strong_count(&a) == 1);
        let b = w.upgrade().expect("upgrade of live rc failed");
        assert!(Rc::strong_count(&b) == 2);
        assert!(Rc::strong_count(&a) == 2);
        drop(w);
        drop(a);
        assert!(Rc::strong_count(&b) == 1);
        let c = b.clone();
        assert!(Rc::strong_count(&b) == 2);
        assert!(Rc::strong_count(&c) == 2);
    }

    #[test]
    fn test_weak_count() {
        let a = Rc::new(0);
        assert!(Rc::strong_count(&a) == 1);
        assert!(Rc::weak_count(&a) == 0);
        let w = Rc::downgrade(&a);
        assert!(Rc::strong_count(&a) == 1);
        assert!(Rc::weak_count(&a) == 1);
        drop(w);
        assert!(Rc::strong_count(&a) == 1);
        assert!(Rc::weak_count(&a) == 0);
        let c = a.clone();
        assert!(Rc::strong_count(&a) == 2);
        assert!(Rc::weak_count(&a) == 0);
        drop(c);
    }

    #[test]
    fn weak_counts() {
        assert_eq!(Weak::weak_count(&Weak::<u64>::new()), None);
        assert_eq!(Weak::strong_count(&Weak::<u64>::new()), 0);

        let a = Rc::new(0);
        let w = Rc::downgrade(&a);
        assert_eq!(Weak::strong_count(&w), 1);
        assert_eq!(Weak::weak_count(&w), Some(1));
        let w2 = w.clone();
        assert_eq!(Weak::strong_count(&w), 1);
        assert_eq!(Weak::weak_count(&w), Some(2));
        assert_eq!(Weak::strong_count(&w2), 1);
        assert_eq!(Weak::weak_count(&w2), Some(2));
        drop(w);
        assert_eq!(Weak::strong_count(&w2), 1);
        assert_eq!(Weak::weak_count(&w2), Some(1));
        let a2 = a.clone();
        assert_eq!(Weak::strong_count(&w2), 2);
        assert_eq!(Weak::weak_count(&w2), Some(1));
        drop(a2);
        drop(a);
        assert_eq!(Weak::strong_count(&w2), 0);
        assert_eq!(Weak::weak_count(&w2), Some(1));
        drop(w2);
    }

    #[test]
    fn try_unwrap() {
        let x = Rc::new(3);
        assert_eq!(Rc::try_unwrap(x), Ok(3));
        let x = Rc::new(4);
        let _y = x.clone();
        assert_eq!(Rc::try_unwrap(x), Err(Rc::new(4)));
        let x = Rc::new(5);
        let _w = Rc::downgrade(&x);
        assert_eq!(Rc::try_unwrap(x), Ok(5));
    }

    #[test]
    fn into_from_raw() {
        let x = Rc::new(box "hello");
        let y = x.clone();

        let x_ptr = Rc::into_raw(x);
        drop(y);
        unsafe {
            assert_eq!(**x_ptr, "hello");

            let x = Rc::from_raw(x_ptr);
            assert_eq!(**x, "hello");

            assert_eq!(Rc::try_unwrap(x).map(|x| *x), Ok("hello"));
        }
    }

    #[test]
    fn test_into_from_raw_unsized() {
        use std::fmt::Display;
        use std::string::ToString;

        let rc: Rc<str> = Rc::from("foo");

        let ptr = Rc::into_raw(rc.clone());
        let rc2 = unsafe { Rc::from_raw(ptr) };

        assert_eq!(unsafe { &*ptr }, "foo");
        assert_eq!(rc, rc2);

        let rc: Rc<dyn Display> = Rc::new(123);

        let ptr = Rc::into_raw(rc.clone());
        let rc2 = unsafe { Rc::from_raw(ptr) };

        assert_eq!(unsafe { &*ptr }.to_string(), "123");
        assert_eq!(rc2.to_string(), "123");
    }

    #[test]
    fn get_mut() {
        let mut x = Rc::new(3);
        *Rc::get_mut(&mut x).unwrap() = 4;
        assert_eq!(*x, 4);
        let y = x.clone();
        assert!(Rc::get_mut(&mut x).is_none());
        drop(y);
        assert!(Rc::get_mut(&mut x).is_some());
        let _w = Rc::downgrade(&x);
        assert!(Rc::get_mut(&mut x).is_none());
    }

    #[test]
    fn test_cowrc_clone_make_unique() {
        let mut cow0 = Rc::new(75);
        let mut cow1 = cow0.clone();
        let mut cow2 = cow1.clone();

        assert!(75 == *Rc::make_mut(&mut cow0));
        assert!(75 == *Rc::make_mut(&mut cow1));
        assert!(75 == *Rc::make_mut(&mut cow2));

        *Rc::make_mut(&mut cow0) += 1;
        *Rc::make_mut(&mut cow1) += 2;
        *Rc::make_mut(&mut cow2) += 3;

        assert!(76 == *cow0);
        assert!(77 == *cow1);
        assert!(78 == *cow2);

        // none should point to the same backing memory
        assert!(*cow0 != *cow1);
        assert!(*cow0 != *cow2);
        assert!(*cow1 != *cow2);
    }

    #[test]
    fn test_cowrc_clone_unique2() {
        let mut cow0 = Rc::new(75);
        let cow1 = cow0.clone();
        let cow2 = cow1.clone();

        assert!(75 == *cow0);
        assert!(75 == *cow1);
        assert!(75 == *cow2);

        *Rc::make_mut(&mut cow0) += 1;

        assert!(76 == *cow0);
        assert!(75 == *cow1);
        assert!(75 == *cow2);

        // cow1 and cow2 should share the same contents
        // cow0 should have a unique reference
        assert!(*cow0 != *cow1);
        assert!(*cow0 != *cow2);
        assert!(*cow1 == *cow2);
    }

    #[test]
    fn test_cowrc_clone_weak() {
        let mut cow0 = Rc::new(75);
        let cow1_weak = Rc::downgrade(&cow0);

        assert!(75 == *cow0);
        assert!(75 == *cow1_weak.upgrade().unwrap());

        *Rc::make_mut(&mut cow0) += 1;

        assert!(76 == *cow0);
        assert!(cow1_weak.upgrade().is_none());
    }

    #[test]
    fn test_show() {
        let foo = Rc::new(75);
        assert_eq!(format!("{:?}", foo), "75");
    }

    #[test]
    fn test_unsized() {
        let foo: Rc<[i32]> = Rc::new([1, 2, 3]);
        assert_eq!(foo, foo.clone());
    }

    #[test]
    fn test_from_owned() {
        let foo = 123;
        let foo_rc = Rc::from(foo);
        assert!(123 == *foo_rc);
    }

    #[test]
    fn test_new_weak() {
        let foo: Weak<usize> = Weak::new();
        assert!(foo.upgrade().is_none());
    }

    #[test]
    fn test_ptr_eq() {
        let five = Rc::new(5);
        let same_five = five.clone();
        let other_five = Rc::new(5);

        assert!(Rc::ptr_eq(&five, &same_five));
        assert!(!Rc::ptr_eq(&five, &other_five));
    }

    #[test]
    fn test_from_str() {
        let r: Rc<str> = Rc::from("foo");

        assert_eq!(&r[..], "foo");
    }

    #[test]
    fn test_copy_from_slice() {
        let s: &[u32] = &[1, 2, 3];
        let r: Rc<[u32]> = Rc::from(s);

        assert_eq!(&r[..], [1, 2, 3]);
    }

    #[test]
    fn test_clone_from_slice() {
        #[derive(Clone, Debug, Eq, PartialEq)]
        struct X(u32);

        let s: &[X] = &[X(1), X(2), X(3)];
        let r: Rc<[X]> = Rc::from(s);

        assert_eq!(&r[..], s);
    }

    #[test]
    #[should_panic]
    fn test_clone_from_slice_panic() {
        use std::string::{String, ToString};

        struct Fail(u32, String);

        impl Clone for Fail {
            fn clone(&self) -> Fail {
                if self.0 == 2 {
                    panic!();
                }
                Fail(self.0, self.1.clone())
            }
        }

        let s: &[Fail] = &[
            Fail(0, "foo".to_string()),
            Fail(1, "bar".to_string()),
            Fail(2, "baz".to_string()),
        ];

        // Should panic, but not cause memory corruption
        let _r: Rc<[Fail]> = Rc::from(s);
    }

    #[test]
    fn test_from_box() {
        let b: Box<u32> = box 123;
        let r: Rc<u32> = Rc::from(b);

        assert_eq!(*r, 123);
    }

    #[test]
    fn test_from_box_str() {
        use std::string::String;

        let s = String::from("foo").into_boxed_str();
        let r: Rc<str> = Rc::from(s);

        assert_eq!(&r[..], "foo");
    }

    #[test]
    fn test_from_box_slice() {
        let s = vec![1, 2, 3].into_boxed_slice();
        let r: Rc<[u32]> = Rc::from(s);

        assert_eq!(&r[..], [1, 2, 3]);
    }

    #[test]
    fn test_from_box_trait() {
        use std::fmt::Display;
        use std::string::ToString;

        let b: Box<dyn Display> = box 123;
        let r: Rc<dyn Display> = Rc::from(b);

        assert_eq!(r.to_string(), "123");
    }

    #[test]
    fn test_from_box_trait_zero_sized() {
        use std::fmt::Debug;

        let b: Box<dyn Debug> = box ();
        let r: Rc<dyn Debug> = Rc::from(b);

        assert_eq!(format!("{:?}", r), "()");
    }

    #[test]
    fn test_from_vec() {
        let v = vec![1, 2, 3];
        let r: Rc<[u32]> = Rc::from(v);

        assert_eq!(&r[..], [1, 2, 3]);
    }

    #[test]
    fn test_downcast() {
        use std::any::Any;

        let r1: Rc<dyn Any> = Rc::new(i32::max_value());
        let r2: Rc<dyn Any> = Rc::new("abc");

        assert!(r1.clone().downcast::<u32>().is_err());

        let r1i32 = r1.downcast::<i32>();
        assert!(r1i32.is_ok());
        assert_eq!(r1i32.unwrap(), Rc::new(i32::max_value()));

        assert!(r2.clone().downcast::<i32>().is_err());

        let r2str = r2.downcast::<&'static str>();
        assert!(r2str.is_ok());
        assert_eq!(r2str.unwrap(), Rc::new("abc"));
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<T: ?Sized> borrow::Borrow<T> for Share<T> {
    fn borrow(&self) -> &T {
        &**self
    }
}

#[stable(since = "1.5.0", feature = "smart_ptr_as_ref")]
impl<T: ?Sized> AsRef<T> for Share<T> {
    fn as_ref(&self) -> &T {
        &**self
    }
}

#[stable(feature = "pin", since = "1.33.0")]
impl<T: ?Sized> Unpin for Share<T> { }


