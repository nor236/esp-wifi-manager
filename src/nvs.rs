use core::f32::consts::E;

use alloc::{rc::Rc, string::String};
use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex,
    mutex::Mutex,
    semaphore::{GreedySemaphore, Semaphore},
};
use embedded_storage::nor_flash::NorFlash as _;
use embedded_storage::{ReadStorage, Storage};
use esp_bootloader_esp_idf::partitions;
use esp_nvs::{error::Error, platform::EspFlash, Key};
use esp_storage::FlashStorage;
use portable_atomic::AtomicU8;
use tickv::{ErrorCode, FlashController};

use crate::{structs::AutoSetupSettings, WmError};
const PART_OFFSET: u32 = 0x8000;
const PART_SIZE: u32 = 0xc00;

static mut NVS_READ_BUF: &mut [u8; 1024] = &mut [0; 1024];
static NVS_INSTANCES: AtomicU8 = AtomicU8::new(0);
pub const NAMESPACE_WIFI: &str = "WIFI";
pub const KEY_SSID: &str = "SSID";
pub const KEY_PASSWORD: &str = "PASSWORD";
pub struct NvsWifiHelper {
    nvs: esp_nvs::Nvs<'static, EspFlash<'static>>,
    nvs_partition: partitions::FlashRegion<'static, FlashStorage<'static>>,
}

impl NvsWifiHelper {
    pub fn new(flash_per: esp_hal::peripherals::FLASH<'static>) -> Self {
        use static_cell::StaticCell;
        static SOME_FLASH: StaticCell<FlashStorage<'static>> = StaticCell::new();

        let flash = esp_storage::FlashStorage::new(unsafe { flash_per.clone_unchecked() });
        // // Initialize it at runtime. This returns a `&'static mut`.
        let flash: &'static mut FlashStorage<'static> = SOME_FLASH.init(flash);

        static SOME_PT_MEM: StaticCell<[u8; partitions::PARTITION_TABLE_MAX_LEN]> =
            StaticCell::new();

        let pt_mem = [0u8; partitions::PARTITION_TABLE_MAX_LEN];
        let pt_mem = SOME_PT_MEM.init(pt_mem);

        let pt = partitions::read_partition_table(flash, pt_mem).unwrap();

        let nvs = pt
            .find_partition(partitions::PartitionType::Data(
                partitions::DataPartitionSubType::Nvs,
            ))
            .unwrap()
            .unwrap();
        let nvs_partition: partitions::FlashRegion<'_, FlashStorage<'_>> =
            nvs.as_embedded_storage(flash);

        let partition_offset = nvs.offset() as usize;
        let partition_size = nvs.len() as usize;
        let storage = esp_storage::FlashStorage::new(unsafe { flash_per.clone_unchecked() });
        static SOME_ESP_FLASH: StaticCell<EspFlash<'static>> = StaticCell::new();
        let esp_flash = EspFlash::new(storage);
        let esp_flash = SOME_ESP_FLASH.init(esp_flash);

        let nvs: esp_nvs::Nvs<'_, EspFlash<'_>> =
            esp_nvs::Nvs::new(partition_offset, partition_size, esp_flash)
                .expect("failed to create nvs");
        Self { nvs, nvs_partition }

        // unsafe { Self::new_unchecked(flash_offset, flash_size, flash) }
    }

    pub fn clear(&mut self) {
        match self.nvs_partition.erase(0, 1024) {
            Ok(_) => {
                log::info!("nvs partition erased");
            }
            Err(e) => {
                log::error!("nvs partition erase failed: {e}");
            }
        }
    }
    pub fn delete(&mut self, namespace: &str, key: &str) -> crate::Result<()> {
        self.nvs
            .delete(&Key::from_str(namespace), &Key::from_str(key))
            .map_err(|_| crate::WmError::NvsError)
    }
    pub fn set_str(&mut self, namespace: &str, key: &str, value: &str) -> crate::Result<()> {
        self.nvs
            .set(&Key::from_str(namespace), &Key::from_str(key), value)
            .map_err(|_| crate::WmError::NvsError)
    }
    pub fn set_bool(&mut self, namespace: &str, key: &str, value: bool) -> crate::Result<()> {
        self.nvs
            .set(&Key::from_str(namespace), &Key::from_str(key), value)
            .map_err(|_| crate::WmError::NvsError)
    }
    pub fn set_i32(&mut self, namespace: &str, key: &str, value: i32) -> crate::Result<()> {
        self.nvs
            .set(&Key::from_str(namespace), &Key::from_str(key), value)
            .map_err(|_| crate::WmError::NvsError)
    }
    pub fn set_u32(&mut self, namespace: &str, key: &str, value: u32) -> crate::Result<()> {
        self.nvs
            .set(&Key::from_str(namespace), &Key::from_str(key), value)
            .map_err(|_| crate::WmError::NvsError)
    }
    pub fn get_str(&mut self, namespace: &str, key: &str) -> Result<String, Error> {
        let rs = self
            .nvs
            .get::<String>(&Key::from_str(namespace), &Key::from_str(key))?;
        Ok(rs)
    }
    pub fn get_bool(&mut self, namespace: &str, key: &str) -> Result<bool, Error> {
        let rs = self
            .nvs
            .get::<bool>(&Key::from_str(namespace), &Key::from_str(key))?;
        Ok(rs)
    }
    pub fn get_i32(&mut self, namespace: &str, key: &str) -> Result<i32, Error> {
        let rs = self
            .nvs
            .get::<i32>(&Key::from_str(namespace), &Key::from_str(key))?;
        Ok(rs)
    }
    pub fn get_u32(&mut self, namespace: &str, key: &str) -> Result<u32, Error> {
        let rs = self
            .nvs
            .get::<u32>(&Key::from_str(namespace), &Key::from_str(key))?;
        Ok(rs)
    }
}

pub fn hash(buf: &[u8]) -> u64 {
    let mut tmp = 0;
    for b in buf {
        tmp ^= *b as u64;
        tmp <<= 1;
    }

    tmp
}

pub fn create_nvs(
    flash_per: esp_hal::peripherals::FLASH<'static>,
) -> esp_nvs::Nvs<'static, EspFlash<'static>> {
    use static_cell::StaticCell;
    static SOME_FLASH: StaticCell<FlashStorage<'static>> = StaticCell::new();

    let flash = esp_storage::FlashStorage::new(unsafe { flash_per.clone_unchecked() });
    // // Initialize it at runtime. This returns a `&'static mut`.
    let flash: &'static mut FlashStorage<'static> = SOME_FLASH.init(flash);

    static SOME_PT_MEM: StaticCell<[u8; partitions::PARTITION_TABLE_MAX_LEN]> = StaticCell::new();

    let pt_mem = [0u8; partitions::PARTITION_TABLE_MAX_LEN];
    let pt_mem = SOME_PT_MEM.init(pt_mem);

    let pt = partitions::read_partition_table(flash, pt_mem).unwrap();

    let nvs = pt
        .find_partition(partitions::PartitionType::Data(
            partitions::DataPartitionSubType::Nvs,
        ))
        .unwrap()
        .unwrap();
    let nvs_partition: partitions::FlashRegion<'_, FlashStorage<'_>> =
        nvs.as_embedded_storage(flash);

    let partition_offset = nvs.offset() as usize;
    let partition_size = nvs.len() as usize;
    let storage = esp_storage::FlashStorage::new(unsafe { flash_per.clone_unchecked() });
    static SOME_ESP_FLASH: StaticCell<EspFlash<'static>> = StaticCell::new();
    let esp_flash = EspFlash::new(storage);
    let esp_flash = SOME_ESP_FLASH.init(esp_flash);

    let nvs: esp_nvs::Nvs<'_, EspFlash<'_>> =
        esp_nvs::Nvs::new(partition_offset, partition_size, esp_flash)
            .expect("failed to create nvs");
    nvs
    // unsafe { Self::new_unchecked(flash_offset, flash_size, flash) }
}
