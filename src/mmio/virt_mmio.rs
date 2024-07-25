//
// virt mmio 仮想デバイス
//

const VIRT_MMIO: usize = 0xa000000;
const VIRT_MMIO_SIZE: usize = 0x200;

const VIRT_MMIO_MAGIC_VALUE: u32 = 0x74726976;
const VIRT_MMIO_VERSION: u32 = 0x2;

const VIRT_MMIO_QUEUE_NUM_MAX: u32 = 1024;

const VIRTIO_RESERVED: u32 = 0x00;
const VIRTIO_NETWORK_CARD: u32 = 0x01;
const VIRTIO_BLOCK_DEVICE: u32 = 0x02;

const VIRT_MMIO_MAGIC_OFFSET: usize = 0x00;
const VIRT_MMIO_VERSION_OFFSET: usize = 0x04;
const VIRT_MMIO_DEVICE_ID_OFFSET: usize = 0x08;
const VIRT_MMIO_VENDOR_ID_OFFSET: usize = 0x0c;
const VIRT_MMIO_DEVICE_FEATURES_OFFSET: usize = 0x10;
const VIRT_MMIO_DEVICE_FEATURE_SEL_OFFSET: usize = 0x14;
const VIRT_MMIO_DRIVER_FEATURES_OFFSET: usize = 0x20;
const VIRT_MMIO_DRIVER_FEATURES_SEL_OFFSET: usize = 0x24;
const VIRT_MMIO_QUEUE_SEL_OFFSET: usize = 0x30;
const VIRT_MMIO_QUEUE_NUM_MAX_OFFSET: usize = 0x34;
const VIRT_MMIO_QUEUE_NUM_OFFSET: usize = 0x38;
const VIRT_MMIO_QUEUE_READY_OFFSET: usize = 0x44;

const VIRT_MMIO_QUEUE_DESC_LOW_OFFSET: usize = 0x80;
const VIRT_MMIO_QUEUE_DESC_HIGH_OFFSET: usize = 0x84;
const VIRT_MMIO_QUEUE_DRIVER_LOW_OFFSET: usize = 0x90;
const VIRT_MMIO_QUEUE_DRIVER_HIGH_OFFSET: usize = 0x94;
const VIRT_MMIO_QUEUE_DEVICE_LOW_OFFSET: usize = 0xa0;
const VIRT_MMIO_QUEUE_DEVICE_HIGH_OFFSET: usize = 0xa4;

pub fn virt_mmio_read(device_type: u32, offset: usize, _access_width: u64) -> Result<u32, ()> {
    match offset {
        VIRT_MMIO_MAGIC_OFFSET => Ok(VIRT_MMIO_MAGIC_VALUE),
        VIRT_MMIO_VERSION_OFFSET => Ok(0x2),
        VIRT_MMIO_DEVICE_ID_OFFSET => Ok(device_type),
        VIRT_MMIO_QUEUE_NUM_MAX_OFFSET => Ok(VIRT_MMIO_QUEUE_NUM_MAX),
        _ => Err(()),
    }
}

pub fn virt_mmio_write(
    device_type: u32,
    offset: usize,
    _access_width: u64,
    value: u32,
) -> Result<(), ()> {
    match offset {
        VIRT_MMIO_QUEUE_READY => {
            // TODO: 対象のvirt queueを有効にする
            Ok(())
        }
        _ => Err(()),
    }
}
