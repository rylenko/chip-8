/// Used by the `Emulator` to run the instructions of the loaded ROM in the
/// `crate::ram::Ram`
///
/// # Chip-8 Registers
///
/// Chip-8 has 16 general purpose 8-bit registers, usually referred to as
/// `self.v`x, where x is a hexadecimal digit (0 through F). There is also a
/// 16-bit register called `self.i`. This register is generally used to store
/// memory addresses, so only the lowest (rightmost) 12 bits are usually used.
///
/// The program counter (`self.pc`) is used to store the currently executing
/// address.
///
/// The `self.return_stack` is used to store the address that the interpreter
/// shoud return to when finished with a subroutine.
pub struct Cpu {
	v: [u8; 16],
	i: u16,
	pc: u16,
	return_stack: Vec<u16>,
	rng: rand::rngs::ThreadRng,
	last_instruction_time: std::time::Instant,
}

impl Cpu {
	#[must_use]
	pub fn new() -> Self {
		Self {
			i: 0,
			pc: crate::consts::RAM_ROM_START_ADDRESS,
			v: [0; 16],
			return_stack: vec![],
			rng: rand::thread_rng(),
			last_instruction_time: std::time::Instant::now(),
		}
	}

	/// Runs the following instruction. Be sure to check
	/// [`self.can_run_instruction`] before running it.
	///
	/// # Debug panic
	///
	/// If `self.can_run_instruction()` is `false`.
	#[allow(clippy::too_many_lines)]
	#[cfg_attr(
		feature = "tracing",
		tracing::instrument(
			fields(
				i = self.i,
				pc = self.pc,
			),
			level = tracing::Level::TRACE,
			skip_all,
		),
	)]
	pub fn run_instruction(
		&mut self,
		ram: &mut crate::ram::Ram,
		timer: &mut crate::timer::Timer,
		screen: &mut crate::screen::Screen,
		keyboard: &crate::keyboard::Keyboard,
	) {
		use rand::Rng;

		debug_assert!(self.can_run_instruction());
		self.last_instruction_time = std::time::Instant::now();

		let first_byte = u16::from(ram.read(self.pc));
		let second_byte = u16::from(ram.read(self.pc + 1));
		let instruction = (first_byte << 8) | second_byte;
		#[cfg(feature = "tracing")]
		tracing::trace!("Read instruction {:#X}:{:#X}", self.pc, instruction);

		// The lowest X bits of the instruction:
		let nnn = instruction & 0x0FFF; // 12
		let nn = (instruction & 0x00FF) as u8; // 8
		let n = (instruction & 0x000F) as u8; // 4

		// The lower 4 bits of the high byte of the instruction
		let x = ((instruction & 0x0F00) >> 8) as u8;
		// The upper 4 bits of the low byte of the instruction
		let y = ((instruction & 0x00F0) >> 4) as u8;

		let xu = x as usize;
		let yu = y as usize;

		match ((instruction & 0xF000) >> 12, nn, n) {
			// Clear the screen
			(0x0, 0xE0, _) => {
				screen.clear();
				self.pc += 2;
			}
			// Return from a subroutine
			(0x0, 0xEE, _) => self.pc = self.return_stack.pop().unwrap(),
			// Jump to location `nnn`
			(0x1, _, _) => self.pc = nnn,
			// Call subroutine at nnn
			(0x2, _, _) => {
				self.return_stack.push(self.pc + 2);
				self.pc = nnn;
			}
			// Skip next instruction if vx == nn
			(0x3, _, _) => {
				if self.v[xu] == nn {
					self.pc += 4;
				} else {
					self.pc += 2;
				}
			}
			// Skip next instruction if vx != nn
			(0x4, _, _) => {
				if self.v[xu] == nn {
					self.pc += 2;
				} else {
					self.pc += 4;
				}
			}
			// Skip next instruction if vx == vy
			(0x5, _, 0x0) => {
				if self.v[xu] == self.v[yu] {
					self.pc += 4;
				} else {
					self.pc += 2;
				}
			}
			// Set vx = nn
			(0x6, _, _) => {
				self.v[xu] = nn;
				self.pc += 2;
			}
			// Set vx = vx + nn
			(0x7, _, _) => {
				self.v[xu] = self.v[xu].wrapping_add(nn);
				self.pc += 2;
			}
			// Set vx = vy
			(0x8, _, 0x0) => {
				self.v[xu] = self.v[yu];
				self.pc += 2;
			}
			// Set vx = vx OR vy
			(0x8, _, 0x1) => {
				self.v[xu] |= self.v[yu];
				self.pc += 2;
			}
			// Set vx = vx AND vy
			(0x8, _, 0x2) => {
				self.v[xu] &= self.v[yu];
				self.pc += 2;
			}
			// Set vx = vx XOR vy
			(0x8, _, 0x3) => {
				self.v[xu] ^= self.v[yu];
				self.pc += 2;
			}
			// Set vx = vx + vy. If overflowing, vf = 1, otherwise vf = 0
			(0x8, _, 0x4) => {
				let (sum, is_overflow) =
					self.v[xu].overflowing_add(self.v[yu]);

				self.v[xu] = sum;
				self.v[0xF] = u8::from(is_overflow);

				self.pc += 2;
			}
			// If vx => vy, vf = 1, otherwise vf = 0. Set vx = vx - vy
			(0x8, _, 0x5) => {
				let (diff, is_overflow) =
					self.v[xu].overflowing_sub(self.v[yu]);

				self.v[xu] = diff;
				self.v[0xF] = u8::from(!is_overflow);

				self.pc += 2;
			}
			// If the least-significant bit of vx is 1, vf = 1, otherwise vf =
			// 0. Set vx = vx SHR 1
			(0x8, _, 0x6) => {
				self.v[0xF] = self.v[xu] & 0x1;
				self.v[xu] >>= 1;

				self.pc += 2;
			}
			// if vx <= vy, vf = 1, otherwise vf = 0. Set vx = vy - vx
			(0x8, _, 0x7) => {
				let (diff, is_overflow) =
					self.v[yu].overflowing_sub(self.v[xu]);

				self.v[xu] = diff;
				self.v[0xF] = u8::from(!is_overflow);

				self.pc += 2;
			}
			// If the most-significant bit of vx is 1, vf = 1, otherwise vf =
			// 0. Set vx = vx SHL 1
			(0x8, _, 0xE) => {
				self.v[0xF] = (self.v[xu] & 0x80) >> 7;
				self.v[xu] <<= 1;

				self.pc += 2;
			}
			// Skip next instruction, if vx != vy
			(0x9, _, 0x0) => {
				if self.v[xu] == self.v[yu] {
					self.pc += 2;
				} else {
					self.pc += 4;
				}
			}
			// Set i = nnn
			(0xA, _, _) => {
				self.i = nnn;
				self.pc += 2;
			}
			// Jump to location nnn + v0
			(0xB, _, _) => self.pc = nnn + u16::from(self.v[0x0]),
			// Set vx = random byte AND nn
			(0xC, _, _) => {
				self.v[xu] = self.rng.gen::<u8>() & nn;
				self.pc += 2;
			}
			// Draws n-byte sprite starting at memory location i at (vx, vy)
			// Sprites are XORed onto the existing screen. If this causes any
			// pixels to be erased, vf = 1, otherwise vf = 0
			(0xD, _, _) => {
				self.draw_sprite(self.v[xu], self.v[yu], n, ram, screen);
				self.pc += 2;
			}
			// Skip next instruction, if vx key is pressed
			(0xE, 0x9E, _) => {
				if keyboard.is_key_pressed(self.v[xu]) {
					self.pc += 4;
				} else {
					self.pc += 2;
				}
			}
			// Skip next instruction, if vx key is not pressed
			(0xE, 0xA1, _) => {
				if keyboard.is_key_pressed(self.v[xu]) {
					self.pc += 2;
				} else {
					self.pc += 4;
				}
			}
			// Set vx = delay timer value
			(0xF, 0x07, _) => {
				self.v[xu] = timer.get_delay();
				self.pc += 2;
			}
			// If any key is pressed, place it code in vx
			(0xF, 0x0A, _) => {
				if let Some(c) = keyboard.pressed_key_code {
					self.v[xu] = c;
					self.pc += 2;
				}
			}
			// Set delay timer = vx
			(0xF, 0x15, _) => {
				timer.set_delay(self.v[xu]);
				self.pc += 2;
			}
			// Set sound timer = vx
			(0xF, 0x18, _) => self.pc += 2, // No sound
			// Set i = i + vx
			(0xF, 0x1E, _) => {
				self.i += u16::from(self.v[xu]);
				self.pc += 2;
			}
			// Set i = location of sprite for digit vx
			(0xF, 0x29, _) => {
				// Multiply by 5 because each sprite has 5 lines, each line is
				// 1 byte.
				self.i = u16::from(self.v[xu]) * 5;
				self.pc += 2;
			}
			// Takes hundreds, tens and ones of vx and writes them one after
			// another starting with i
			(0xF, 0x33, _) => {
				let vx = self.v[xu];
				ram.write(self.i, vx / 100);
				ram.write(self.i + 1, (vx % 100) / 10);
				ram.write(self.i + 2, vx % 10);
				self.pc += 2;
			}
			// Store registers v0 through vx im memory starting at location i
			(0xF, 0x55, _) => {
				for i in 0..=x {
					ram.write(self.i + u16::from(i), self.v[i as usize]);
				}
				self.pc += 2;
			}
			// Read register v0 through vx from memory starting at location i
			(0xF, 0x65, _) => {
				for i in 0..=x {
					self.v[i as usize] = ram.read(self.i + u16::from(i));
				}
				self.pc += 2;
			}
			_ => unreachable!(
				"Invalid instruction: {:#X}:{:#X}",
				self.pc, instruction
			),
		}
	}

	/// Determines whether enough time has elapsed for us to run a new
	/// instruction
	#[inline]
	#[must_use]
	pub fn can_run_instruction(&self) -> bool {
		self.last_instruction_time.elapsed()
			> std::time::Duration::from_millis(2)
	}

	/// Draws `length`-byte sprite starting at memory location `self.i` at `x`,
	/// `y` using [`crate::.draw_byte`].
	#[cfg_attr(
		feature = "tracing",
		tracing::instrument(
			fields(
				i = self.i,
				pc = self.pc,
			),
			level = tracing::Level::TRACE,
			skip(self, ram, screen),
		),
	)]
	#[inline]
	fn draw_sprite(
		&mut self,
		x: u8,
		y: u8,
		length: u8,
		ram: &crate::ram::Ram,
		screen: &mut crate::screen::Screen,
	) {
		let mut should_set_vf = false;

		for sprite_i in 0..length {
			let byte = ram.read(self.i + u16::from(sprite_i));
			let is_erased =
				screen.draw_byte(byte, x as usize, (y + sprite_i) as usize);

			if is_erased {
				should_set_vf = true;
			}
		}

		self.v[0xF] = u8::from(should_set_vf);
	}
}
