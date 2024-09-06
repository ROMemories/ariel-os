#[doc(alias = "master")]
pub mod main;

use embassy_rp::spi::{Phase, Polarity};
use riot_rs_shared_types::spi::Mode;

fn from_mode(mode: Mode) -> (Polarity, Phase) {
    match mode {
        Mode::Mode0 => (Polarity::IdleLow, Phase::CaptureOnFirstTransition),
        Mode::Mode1 => (Polarity::IdleLow, Phase::CaptureOnSecondTransition),
        Mode::Mode2 => (Polarity::IdleHigh, Phase::CaptureOnFirstTransition),
        Mode::Mode3 => (Polarity::IdleHigh, Phase::CaptureOnSecondTransition),
    }
}
