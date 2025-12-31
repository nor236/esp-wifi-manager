use esp_bootloader_esp_idf::partitions;
use esp_nvs::{error::Error, platform::EspFlash};

pub fn new_nvs(
    flash_per: esp_hal::peripherals::FLASH<'static>,
) -> Result<esp_nvs::Nvs<'static, EspFlash<'static>>, Error> {
    use static_cell::StaticCell;
    let mut flash = esp_storage::FlashStorage::new(unsafe { flash_per.clone_unchecked() });
    let mut pt_mem = [0u8; partitions::PARTITION_TABLE_MAX_LEN];
    let pt = partitions::read_partition_table(&mut flash, &mut pt_mem).unwrap();
    let nvs = pt
        .find_partition(partitions::PartitionType::Data(
            partitions::DataPartitionSubType::Nvs,
        ))
        .unwrap()
        .unwrap();
    let partition_offset = nvs.offset() as usize;
    let partition_size = nvs.len() as usize;
    let storage = esp_storage::FlashStorage::new(unsafe { flash_per.clone_unchecked() });
    static SOME_ESP_FLASH: StaticCell<EspFlash<'static>> = StaticCell::new();
    let esp_flash = EspFlash::new(storage);
    let esp_flash = SOME_ESP_FLASH.init(esp_flash);
    let nvs: esp_nvs::Nvs<'_, EspFlash<'_>> =
        esp_nvs::Nvs::new(partition_offset, partition_size, esp_flash)?;
    Ok(nvs)
}
