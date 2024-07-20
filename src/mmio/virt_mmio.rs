//
// virt mmio 仮想デバイス
//

const VIRT_MMIO: usize = 0xa000000;
const VIRT_MMIO_SIZE: usize = 0x200;

const VIRT_MMIO_MAGIC_VALUE: u32 = 0x74726976;
const VIRT_MMIO_VERSION: u32 = 0x2;

const VIRTIO_RESERVED: u32 = 0x00;
const VIRTIO_NETWORK_CARD: u32 = 0x01;
const VIRTIO_BLOCK_DEVICE: u32 = 0x02;

const VIRT_MMIO_MAGIC_OFFSET: usize = 0x00;
const VIRT_MMIO_VERSION_OFFSET: usize = 0x04;
const VIRT_MMIO_DEVICE_ID_OFFSET: usize = 0x08;
const VIRT_MMIO_VENDOR_ID_OFFSET: usize = 0x0c;

pub fn virt_mmio_read(device_type: u32, offset: usize, _access_width: u64) -> Result<u32, ()>
{
    match offset {
        VIRT_MMIO_MAGIC_OFFSET => Ok(VIRT_MMIO_MAGIC_VALUE),
        VIRT_MMIO_VERSION_OFFSET => Ok(0x2),
        VIRT_MMIO_DEVICE_ID_OFFSET => Ok(device_type),
        _ => {
            Err(())
        }
    }
}

pub fn virt_mmio_write(device_type: u32, offset: usize, _access_width: u64, value: u32) -> Result<(), ()>
{
    
}
