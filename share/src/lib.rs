#[cfg(feature = "rc")]
use std::rc::{Rc, Weak};
#[cfg(feature = "rc")]
pub type Share<T> = Rc<T>;
#[cfg(feature = "rc")]
pub type ShareWeak<T> = Weak<T>;

#[cfg(feature = "arc")]
use std::sync::{Arc, Weak};
#[cfg(feature = "arc")]
pub type Share<T> = Arc<T>;
#[cfg(feature = "arc")]
pub type ShareWeak<T> = Weak<T>;