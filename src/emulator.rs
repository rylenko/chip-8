use anyhow::Result;

/// The `Emulator` is an assembler and initializer for all important
/// components: `Cpu`, `Ram`, `Timer`, `Screen`, `Keyboard`.
pub struct Emulator {
	cpu: crate::cpu::Cpu,
	ram: crate::ram::Ram,
	timer: crate::timer::Timer,
	screen: crate::screen::Screen,
	keyboard: crate::keyboard::Keyboard,
}

impl Emulator {
	#[must_use]
	pub fn new() -> Self {
		let mut ram = crate::ram::Ram::new();
		ram.load_digit_sprites();

		Self {
			cpu: crate::cpu::Cpu::new(),
			ram,
			timer: crate::timer::Timer::new(),
			screen: crate::screen::Screen::new(),
			keyboard: crate::keyboard::Keyboard::new(),
		}
	}

	#[inline]
	pub fn load_rom(&mut self, data: &[u8]) {
		self.ram.load_rom(data);
	}

	#[inline]
	pub fn press_key(&mut self, code: u8) {
		self.keyboard.press_key(code);
	}

	#[inline]
	pub fn reset_pressed_key(&mut self) {
		self.keyboard.reset_pressed_key();
	}

	#[inline]
	#[must_use]
	pub fn can_reset_pressed_key(&self) -> bool {
		self.keyboard.can_reset_pressed_key()
	}

	#[inline]
	pub fn run_instruction(&mut self) {
		self.cpu.run_instruction(
			&mut self.ram,
			&mut self.timer,
			&mut self.screen,
			&self.keyboard,
		);
	}

	#[inline]
	#[must_use]
	pub fn can_run_instruction(&self) -> bool {
		self.cpu.can_run_instruction()
	}

	#[inline]
	pub fn display(&mut self, window: &mut minifb::Window) -> Result<()> {
		self.screen.display(window)
	}

	#[inline]
	#[must_use]
	pub fn can_display(&self) -> bool {
		self.screen.can_display()
	}
}
