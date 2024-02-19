/// Delay timer for the `crate::emulator::Emulator`.
///
/// You can set the delay with `self.set_delay` and get the remaining delay
/// with `self.get_delay`.
pub struct Timer {
	delay: u8,
	delay_set_time: std::time::Instant,
}

impl Timer {
	#[must_use]
	pub fn new() -> Self {
		Self { delay: 0, delay_set_time: std::time::Instant::now() }
	}

	/// If the required number of ticks passes, returns `0`, otherwise it
	/// returns the remaining ticks.
	#[cfg_attr(
		feature = "tracing",
		tracing::instrument(level = tracing::Level::TRACE, ret, skip(self)),
	)]
	#[must_use]
	pub fn get_delay(&self) -> u8 {
		use std::convert::TryFrom as _;

		let ticks = self.delay_set_time.elapsed().as_millis() / 16;
		if ticks >= u128::from(self.delay) {
			0
		} else if let Ok(ticks) = u8::try_from(ticks) {
			self.delay - ticks
		} else {
			unreachable!();
		}
	}

	#[cfg_attr(
		feature = "tracing",
		tracing::instrument(level =  tracing::Level::TRACE, skip(self)),
	)]
	pub fn set_delay(&mut self, delay: u8) {
		self.delay = delay;
		self.delay_set_time = std::time::Instant::now();
	}
}
