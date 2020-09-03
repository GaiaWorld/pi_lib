
#[cfg(feature = "web")]
use web_sys::window;

#[cfg(feature = "web")]
pub fn performance_now() -> f32{
	let window = window().expect("should have a window in this context");
	let performance = window
        .performance()
		.expect("performance should be available");
	performance.now() as f32
}






// #[cfg(not(feature = "web"))]
// use std::time::Instant;
#[cfg(not(feature = "web"))]
pub fn performance_now() -> f32{
	0.0 // TODO
	// Instant::now().elapsed().
}
