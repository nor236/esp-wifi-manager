# esp-wifi-manager
forked from [filipton/esp-hal-wifimanager](https://github.com/filipton/esp-hal-wifimanager)，Removed the following dependencies：

- picoserve ，Replace with custom simple http server.
- serde/serde_json  ，Replace with urlencoded.
- tickv ，Replace with esp-nvs

Easy to use Wifimanager for esp-hal (no-std).

If it can't connect to wifi it spawns BLE server (You can use chrome on android or windows to configure it)
and open wifi accesspoint with DHCP server.

## Features (crate)
- `ap` feature that will spawn ap to connect to
- `ble` feature that will spawn ble server to connect to
- `env` feature that will automatically setup wifi from env vars (for quick and easy testing)
- `esp32c3`/`esp32c6`/`esp32s3`/`esp32` feature to select platform
    - other platforms are supported, but i haven't tested them!

If neither `ap`, `ble` nor `env` feature is selected, crate will fail to compile.
Obviously you need to select your platform (`esp32s3` / `esp32c3`)

### How to use env feature
Env feature will automatically setup wifi after startup, to use it:
- Set [env] WM_CONN inside `.cargo/config.toml` file
- Start `cargo run` with WM_CONN env var like this:
```bash
cargo run --config "env.WM_CONN='{\"ssid\": \"ssid\", \"psk\": \"pass\", \"data\": {}}'"
```

## Simple example
Add this to your Cargo.toml (note also add `embassy`, its only for async):

NOTE: this section is not updated, will update it sometime near feature.
```toml
[dependencies]
esp-hal = { version = "1.0.0", features = [ "esp32c3", "unstable" ] }
esp-wifi = { version = "0.15.0", features = [ "esp32c3", "coex" ] }
esp-hal-embassy = { version = "0.9.0", features = ["esp32c3"] }
esp-wifi-manager = { version="0.3.2", default-features = false, features = [
    "esp32c3",
    "ble",
    "ap",
] }
```

Simple example :

```rust
let mut nvs = esp_wifi_manager::nvs::new_nvs(peripherals.FLASH).unwrap();
//esp_wifi_manager::clear_wifi(&mut nvs).unwrap();
let wifi_res =esp_wifi_manager::start_wifi(&spawner, &mut nvs, peripherals.WIFI, peripherals.BT).await;
log::info!("wifi_res: {wifi_res:?}");
```

 


More customized examples:(to see full example check `./examples`-esp32xx dir):

```rust
// ...
let mut nvs = esp_wifi_manager::nvs::new_nvs(peripherals.FLASH).unwrap();
//esp_wifi_manager::clear_wifi(&mut nvs).unwrap();
 
let wifi_res = esp_hal_wifimanager::init_wm(
    wm_settings,
    &spawner,
    &mut nvs,
    rng.clone(),
    peripherals.WIFI,
    peripherals.BT, // only if ble feature is enabled
    None, // signal for ap/ble start
)
.await;
```
