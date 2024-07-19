const VIRT_MMIO: usize = 0xa000000;
const VIRT_MMIO_SIZE: usize = 0x200;

pub fn virt_mmio_read(offset: usize, _access_width: u64) -> Result<u64, ()>
{
    
}

pub fn virt_mmio_write(offset: usize, _access_width: u64, value: u64) -> Result<(), ()>
{

}
