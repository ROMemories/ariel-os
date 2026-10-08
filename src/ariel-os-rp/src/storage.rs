use embassy_rp::{flash::Async, peripherals::FLASH};

pub type Flash = embassy_rp::flash::Flash<'static, FLASH, Async, FLASH_SIZE>;
pub type FlashError = embassy_rp::flash::Error;

const FLASH_SIZE: usize = ariel_os_utils::usize_from_env!("CHIP_NVM_SIZE_BYTES", "flash size");

pub fn init(p: &mut crate::OptionalPeripherals) -> Flash {
    embassy_rp::flash::Flash::<_, Async, FLASH_SIZE>::new(
        p.FLASH.take().unwrap(),
        p.DMA_CH1.take().unwrap(),
    )
}
