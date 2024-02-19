/// Ram of our Chip-8 emulator.
///
/// The first `512` bytes (ending at the `0x200` address) reserved for
/// hexadecimal digits (`consts::RAM_DIGIT_SPRITES`) and the VM implementation.
/// Our ROM we must load starting at `0x200`, which is constant
/// `consts::RAM_PROGRAM_START_ADDRESS`.
pub struct Ram {
	memory: [u8; 4096],
}

impl Ram {
	#[must_use]
	pub const fn new() -> Self {
		Self { memory: [0; 4096] }
	}

	/// Loads `consts::RAM_DIGIT_SPRITES` into the first 80 bytes of memory.
	///
	/// # Debug panic
	///
	/// If sprites already loaded.
	#[cfg_attr(feature = "tracing", tracing::instrument(skip(self)))]
	pub fn load_digit_sprites(&mut self) {
		debug_assert!(self.memory[..80].iter().all(|b| *b == 0));

		let mut address = 0;
		for sprite in &crate::consts::RAM_DIGIT_SPRITES {
			for part in sprite {
				self.write(address, *part);
				address += 1;
			}
		}
	}

	#[cfg_attr(
		feature = "tracing",
		tracing::instrument(level = tracing::Level::TRACE, ret, skip(self)),
	)]
	#[inline]
	#[must_use]
	pub fn read(&self, address: u16) -> u8 {
		self.memory[address as usize]
	}

	#[cfg_attr(
		feature = "tracing",
		tracing::instrument(level = tracing::Level::TRACE, skip(self)),
	)]
	#[inline]
	pub fn write(&mut self, address: u16, value: u8) {
		self.memory[address as usize] = value;
	}

	/// Loads ROM into `self.memory` using `self.write` starting from
	/// `consts::RAM_PROGRAM_START_ADDRESS`.
	#[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
	pub fn load_rom(&mut self, data: &[u8]) {
		use std::convert::TryFrom as _;
		for (i, &byte) in data.iter().enumerate() {
			// `Result::unwrap` because always `i < 4096`
			let address = crate::consts::RAM_ROM_START_ADDRESS
				+ u16::try_from(i).unwrap();
			self.write(address, byte);
		}
	}
}
