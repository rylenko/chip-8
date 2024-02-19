use anyhow::{Context as _, Result};

/// Represents the `emulator::Emulator` screen.
///
/// Stores a `buffer` which contains `0` and `1` for each pixel on the screen.
pub struct Screen {
	buffer: [u8; crate::consts::SCREEN_SIZE],
	last_display_time: std::time::Instant,
}

impl Screen {
	#[must_use]
	pub fn new() -> Self {
		Self {
			buffer: [0; crate::consts::SCREEN_SIZE],
			last_display_time: std::time::Instant::now(),
		}
	}

	/// Clears the screen by setting every bit of `self.buffer` to 0.
	pub fn clear(&mut self) {
		for pixel in &mut self.buffer {
			*pixel = 0;
		}
	}

	/// Displays the `self.buffer` on the [window](minifb::Window).
	///
	/// Since the original screen size is very small, we display it in a large
	/// window by incrementing each pixel by `consts::WINDOW_MULTIPLIER` and
	/// translating `0` and `1` into `consts::BLACK_COLOR` and
	/// `consts::WHITE_COLOR` respectively. All this is stored in
	/// `window_argb_buffer` variable.
	#[cfg_attr(
		feature = "tracing",
		tracing::instrument(level = tracing::Level::TRACE, skip_all),
	)]
	#[inline]
	pub fn display(&mut self, window: &mut minifb::Window) -> Result<()> {
		assert!(self.can_display());

		let mut window_buffer =
			vec![0; crate::consts::WINDOW_SIZE].into_boxed_slice();

		for window_y in 0..crate::consts::WINDOW_HEIGHT {
			let y = window_y / crate::consts::WINDOW_MULTIPLIER;

			for window_x in 0..crate::consts::WINDOW_WIDTH {
				let x = window_x / crate::consts::WINDOW_MULTIPLIER;

				let buffer_index = y * crate::consts::SCREEN_WIDTH + x;
				let window_buffer_index =
					window_y * crate::consts::WINDOW_WIDTH + window_x;

				let pixel = self.buffer[buffer_index];
				let pixel_color = match pixel {
					0 => crate::consts::BLACK_COLOR,
					1 => crate::consts::WHITE_COLOR,
					_ => unreachable!(),
				};
				window_buffer[window_buffer_index] = pixel_color;
			}
		}

		window
			.update_with_buffer(
				&window_buffer,
				crate::consts::WINDOW_WIDTH,
				crate::consts::WINDOW_HEIGHT,
			)
			.context("Failed to update buffer.")?;
		self.last_display_time = std::time::Instant::now();
		Ok(())
	}

	/// Determines whether enough time has elapsed for us to display new
	/// `self.buffer`
	#[inline]
	#[must_use]
	pub fn can_display(&self) -> bool {
		self.last_display_time.elapsed() > std::time::Duration::from_millis(10)
	}

	/// Draws a byte in the `self.buffer` at `x` and `y` coordinates.
	///
	/// Returns a `bool` that informs if a bit has been erased from the screen
	/// (`self.buffer`).
	#[cfg_attr(
		feature = "tracing",
		tracing::instrument(level = tracing::Level::TRACE, skip(self), ret),
	)]
	pub fn draw_byte(
		&mut self,
		mut byte: u8,
		mut x: usize,
		mut y: usize,
	) -> bool {
		let mut is_erased = false;
		y %= crate::consts::SCREEN_HEIGHT;

		for _ in 0..8 {
			x %= crate::consts::SCREEN_WIDTH;
			let buffer_index = y * crate::consts::SCREEN_WIDTH + x;

			let previous_bit = self.buffer[buffer_index];
			let bit = (byte & 0b1000_0000) >> 7;
			let current_bit = previous_bit ^ bit;

			self.buffer[buffer_index] = current_bit;

			if previous_bit == 1 && current_bit == 0 {
				is_erased = true;
			}

			x += 1;
			byte <<= 1;
		}
		is_erased
	}
}
