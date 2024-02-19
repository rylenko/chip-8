/// Chip-8 `Keyboard` for the `Emulator`.
///
/// Since the structure works with `u8` key codes, you should get pressed key
/// ([`minifb::Key`]) code with `Self::get_key_code` before using
/// `self.press_key_code`.
pub struct Keyboard {
	pub pressed_key_code: Option<u8>,
	pressed_key_time: std::time::Instant,
}

impl Keyboard {
	pub fn new() -> Self {
		Self {
			pressed_key_code: None,
			pressed_key_time: std::time::Instant::now(),
		}
	}

	/// Returns the Chip-8 code of the pressed [`minifb::Key`] key.
	#[cfg_attr(
		feature = "tracing",
		tracing::instrument(level = tracing::Level::TRACE, ret),
	)]
	#[inline]
	#[must_use]
	pub fn get_key_code(key: minifb::Key) -> Option<u8> {
		use minifb::Key;
		match key {
			Key::Key1 => Some(0x1),
			Key::Key2 => Some(0x2),
			Key::Key3 => Some(0x3),
			Key::Key4 => Some(0xC),

			Key::Q => Some(0x4),
			Key::W => Some(0x5),
			Key::E => Some(0x6),
			Key::R => Some(0xD),

			Key::A => Some(0x7),
			Key::S => Some(0x8),
			Key::D => Some(0x9),
			Key::F => Some(0xE),

			Key::Z => Some(0xA),
			Key::X => Some(0x0),
			Key::C => Some(0xB),
			Key::V => Some(0xF),

			_ => None,
		}
	}

	/// Needed to register a new keystroke. If you want to null the current
	/// keypress or reset it as invalid, consider `self.reset_pressed_key`.
	#[cfg_attr(tracing, tracing::instrument)]
	#[inline]
	pub fn press_key(&mut self, code: u8) {
		self.pressed_key_code = Some(code);
		self.pressed_key_time = std::time::Instant::now();
	}

	#[cfg_attr(tracing, tracing::instrument(ret))]
	#[inline]
	#[must_use]
	pub fn is_key_pressed(&self, code: u8) -> bool {
		self.pressed_key_code == Some(code)
	}

	/// Resets the currently pressed key. Before using this, make sure to check
	/// `self.can_reset_pressed_key`.
	#[cfg_attr(tracing, tracing::instrument)]
	#[inline]
	pub fn reset_pressed_key(&mut self) {
		assert!(self.can_reset_pressed_key());
		self.pressed_key_code = None;
	}

	/// Determines whether enough time has elapsed for us to reset pressed key.
	#[inline]
	#[must_use]
	pub fn can_reset_pressed_key(&self) -> bool {
		self.pressed_key_code.is_some()
			&& self.pressed_key_time.elapsed()
				>= std::time::Duration::from_millis(200)
	}
}
