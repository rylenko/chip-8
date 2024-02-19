#![deny(clippy::correctness)]
#![warn(
	clippy::complexity,
	clippy::pedantic,
	clippy::perf,
	clippy::style,
	clippy::suspicious
)]
#![allow(
	clippy::as_conversions,
	clippy::implicit_return,
	clippy::missing_docs_in_private_items
)]

mod consts;
mod cpu;
mod emulator;
mod keyboard;
mod ram;
mod screen;
mod timer;

use anyhow::{Context as _, Result};

#[inline]
fn extract_path_from_args() -> Result<std::path::PathBuf> {
	match std::env::args().nth(1) {
		Some(path) => Ok(std::path::PathBuf::from(path)),
		None => Err(anyhow::anyhow!("Enter a path to the ROM.")),
	}
}

#[inline]
fn prepare_emulator(emulator: &mut emulator::Emulator) -> Result<()> {
	// Load rom
	let path = extract_path_from_args()
		.context("Failed to extract path from args.")?;
	let rom_data = std::fs::read(path).context("Failed to read path.")?;
	emulator.load_rom(&rom_data);
	Ok(())
}

#[inline]
fn process_window(
	window: &mut minifb::Window,
	emulator: &mut emulator::Emulator,
) -> Result<()> {
	while window.is_open() && !window.is_key_down(minifb::Key::Escape) {
		// Get pressed key
		let key = match window.get_keys_pressed(minifb::KeyRepeat::Yes) {
			Some(keys) => {
				if keys.is_empty() {
					None
				} else {
					#[allow(clippy::indexing_slicing)]
					Some(keys[0])
				}
			}
			None => None,
		};

		// Check that key is valid
		let mut key_is_valid = false;
		if let Some(k) = key {
			if let Some(c) = keyboard::Keyboard::get_key_code(k) {
				emulator.press_key(c);
				key_is_valid = true;
			}
		}

		// Reset pressed key, run instruction and display
		if emulator.can_reset_pressed_key() && !key_is_valid {
			emulator.reset_pressed_key();
		}
		if emulator.can_run_instruction() {
			emulator.run_instruction();
		}
		if emulator.can_display() {
			emulator.display(window).context("Failed to display.")?;
		}
	}
	Ok(())
}

#[cfg(feature = "tracing")]
#[inline]
fn set_tracing_subscriber(
) -> Result<tracing_appender::non_blocking::WorkerGuard> {
	use tracing_subscriber::layer::SubscriberExt as _;

	// Create the writer
	let (writer, guard) = tracing_appender::non_blocking(
		tracing_appender::rolling::never("", consts::LOGS_FILENAME),
	);

	// Create and set the subscriber
	let subscriber = tracing_subscriber::Registry::default()
		.with(tracing_subscriber::EnvFilter::new(consts::LOG_LEVEL))
		.with(tracing_bunyan_formatter::JsonStorageLayer)
		.with(tracing_bunyan_formatter::BunyanFormattingLayer::new(
			consts::WINDOW_TITLE.to_owned(),
			writer,
		));
	tracing::subscriber::set_global_default(subscriber)
		.context("Failed to set.")?;
	Ok(guard)
}

fn main() -> Result<()> {
	#[cfg(feature = "tracing")]
	let _guard = set_tracing_subscriber()
		.context("Failed to set a tracing subscriber.")?;

	// Create the emulator
	let mut emulator = emulator::Emulator::new();
	prepare_emulator(&mut emulator)
		.context("Failed to prepare the emulator.")?;

	// Create and process a window
	let mut window = minifb::Window::new(
		consts::WINDOW_TITLE,
		consts::WINDOW_WIDTH,
		consts::WINDOW_HEIGHT,
		minifb::WindowOptions::default(),
	)
	.context("Failed to create new window.")?;
	process_window(&mut window, &mut emulator)
		.context("Failed to process a window.")
}
