#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_hal::clock::CpuClock;
use esp_hal::timer::timg::TimerGroup;
use log::info;

extern crate alloc;

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

#[allow(
    clippy::large_stack_frames,
    reason = "it's not unusual to allocate larger buffers etc. in main"
)]
#[esp_rtos::main]
async fn main(spawner: Spawner) -> ! {
    // generator version: 1.1.0

    esp_println::logger::init_logger_from_env();

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    esp_alloc::heap_allocator!(#[esp_hal::ram(reclaimed)] size: 98768);

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_rtos::start(timg0.timer0);
    let rng = esp_hal::rng::Rng::new();
    let mut nvs = esp_wifi_manager::NvsWifiHelper::new(peripherals.FLASH);

    let mut wm_settings = esp_wifi_manager::WmSettings::default();

    wm_settings.ssid.clear();
    _ = core::fmt::write(
        &mut wm_settings.ssid,
        format_args!("ESP-{:X}", esp_wifi_manager::get_efuse_mac()),
    );

    wm_settings.wifi_conn_timeout = 30000;
    wm_settings.esp_reset_timeout = Some(300000); // 5min

    let wifi_res = esp_wifi_manager::init_wm(
        wm_settings,
        &spawner,
        &mut nvs,
        rng.clone(),
        peripherals.WIFI,
        peripherals.BT,
        None,
    )
    .await;

    log::info!("wifi_res: {wifi_res:?}");

    loop {
        info!("Hello world!");
        Timer::after(Duration::from_secs(1)).await;
    }

    // for inspiration have a look at the examples at https://github.com/esp-rs/esp-hal/tree/esp-hal-v~1.0/examples
}
