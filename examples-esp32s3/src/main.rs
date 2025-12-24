#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

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
    esp_alloc::heap_allocator!(size: 150 * 1024);
    let peripherals = esp_hal::init(esp_hal::Config::default());

    /*
    let mut rtc = Rtc::new(peripherals.LPWR, None);
    rtc.rwdt.set_timeout(2.secs());
    rtc.rwdt.enable();
    log::info!("RWDT watchdog enabled!");
    */
    esp_println::logger::init_logger_from_env();

    // let timg0 = TimerGroup::new(peripherals.TIMG0);
    // esp_rtos::start(timg0.timer0, timg0.timer1);

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
        //rtc.rwdt.feed();
        log::info!("bump {}", esp_hal::time::Instant::now());
        Timer::after_millis(15000).await;
    }
}
