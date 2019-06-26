#[cfg(not(feature = "arc"))]
use std::rc::{Rc, Weak};
#[cfg(not(feature = "arc"))]
pub type Share<T> = Rc<T>;
#[cfg(not(feature = "arc"))]
pub type ShareWeak<T> = Weak<T>;

#[cfg(feature = "arc")]
use std::sync::{Arc, Weak};
#[cfg(feature = "arc")]
pub type Share<T> = Arc<T>;
#[cfg(feature = "arc")]
pub type ShareWeak<T> = Weak<T>;