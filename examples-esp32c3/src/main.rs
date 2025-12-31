#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_time::Timer;
use esp_backtrace as _;
use esp_hal::timer::timg::TimerGroup;

/*
// TODO: maybe i should make another crate for this make_static?
/// This is macro from static_cell (static_cell::make_static!) but without weird stuff
macro_rules! make_static {
    ($val:expr) => {{
        type T = impl ::core::marker::Sized;
        static STATIC_CELL: static_cell::StaticCell<T> = static_cell::StaticCell::new();
        STATIC_CELL.uninit().write($val)
    }};
}
*/

esp_bootloader_esp_idf::esp_app_desc!();

#[esp_rtos::main]
async fn main(spawner: Spawner) {
    esp_println::logger::init_logger_from_env();
    let peripherals = esp_hal::init(esp_hal::Config::default());
    esp_alloc::heap_allocator!(#[esp_hal::ram(reclaimed)] size: 64 * 1024);
    esp_alloc::heap_allocator!(size: 36 * 1024);
    let timg0 = TimerGroup::new(peripherals.TIMG0);
    let sw_int =
        esp_hal::interrupt::software::SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);
    esp_rtos::start(timg0.timer0, sw_int.software_interrupt0);

    let mut nvs = esp_wifi_manager::nvs::new_nvs(peripherals.FLASH).unwrap();
    esp_wifi_manager::clear_wifi(&mut nvs).unwrap();

    let wifi_res =
        esp_wifi_manager::start_wifi(&spawner, &mut nvs, peripherals.WIFI, peripherals.BT).await;

    log::info!("wifi_res: {wifi_res:?}");

    loop {
        //rtc.rwdt.feed();
        // log::info!("bump {}", esp_hal::time::Instant::now());
        Timer::after_millis(15000).await;
    }
}
