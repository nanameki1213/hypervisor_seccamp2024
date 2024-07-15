// Copyright (c) 2022 RIKEN
// Copyright (c) 2022 National Institute of Advanced Industrial Science and Technology (AIST)
// All rights reserved.
//
// This software is released under the MIT License.
// http://opensource.org/licenses/mit-license.php

//!
//! Paging
//!
use core::arch::asm;
use crate::cpu::*;


pub unsafe fn eret() -> ! {
    asm!("eret", options(noreturn))
}

pub fn flush_tlb_el1() {
    unsafe {
        asm!(
            "
            dsb ishst
            tlbi alle1is
            "
        );
    }
}

use crate::{allocate_memory, bitmask, cpu::*};

#[derive(Clone, Debug)]
pub struct TableEntry(u64);

#[derive(Clone)]
pub enum Shareability {
    NonShareable = 0b00,
    OuterShareable = 0b10,
    InterShareable = 0b11,
}

pub const PAGE_SHIFT: usize = 12;
pub const PAGE_SIZE: usize = 1 << PAGE_SHIFT;
pub const PAGE_MASK: usize = !(PAGE_SIZE - 1);
pub const STAGE_2_PAGE_SHIFT: usize = 12;
pub const STAGE_2_PAGE_SIZE: usize = 1 << STAGE_2_PAGE_SHIFT;
pub const STAGE_2_PAGE_MASK: usize = !(STAGE_2_PAGE_SIZE - 1);
pub const PAGE_TABLE_SIZE: usize = 0x1000;

pub const PAGE_DESCRIPTORS_UPPER_ATTRIBUTES_OFFSET: u64 = 50;
pub const PAGE_DESCRIPTORS_CONTIGUOUS: u64 = 1 << 52;
pub const PAGE_DESCRIPTORS_NX_BIT_OFFSET: u64 = 54;

pub const PAGE_DESCRIPTORS_NT: u64 = 1 << 16;
pub const PAGE_DESCRIPTORS_AF_BIT_OFFSET: u64 = 10;
pub const PAGE_DESCRIPTORS_AF: u64 = 1 << PAGE_DESCRIPTORS_AF_BIT_OFFSET;
pub const PAGE_DESCRIPTORS_SH_BITS_OFFSET: u64 = 8;
pub const PAGE_DESCRIPTORS_SH_INNER_SHAREABLE: u64 = 0b11 << PAGE_DESCRIPTORS_SH_BITS_OFFSET;
pub const PAGE_DESCRIPTORS_AP_BITS_OFFSET: u64 = 6;

pub const MEMORY_PERMISSION_READABLE_BIT: u8 = 0;
pub const MEMORY_PERMISSION_WRITABLE_BIT: u8 = 1;
pub const MEMORY_PERMISSION_EXECUTABLE_BIT: u8 = 2;

pub const VTTBR_BADDR: u64 = ((1 << 47) - 1) & !1;

impl TableEntry {
    const TABLE_ADDRESS_MASK: u64 = ((1 << 52) - 1) & (PAGE_SIZE as u64 - 1);
    const OUTPUT_ADDRESS_MASK: u64 = ((1 << 50) - 1) & (PAGE_SIZE as u64 - 1);
    const AF_OFFSET: u64 = 10;
    const AF: u64 = 1 << Self::AF_OFFSET;
    const SH_OFFSET: u64 = 8;
    const SH: u64 = 0b11 << Self::SH_OFFSET;
    const S2AP_OFFSET: u64 = 6;
    const S2AP: u64 = 0b11 << Self::S2AP_OFFSET;
    const ATTR_INDEX_OFFSET: u64 = 2;
    const ATTR_INDEX: u64 = 0b1111 << Self::ATTR_INDEX_OFFSET;
    const ATTR_WRITE_BACK: u64 = 0b1111 << Self::ATTR_INDEX_OFFSET;

    pub const fn new() -> Self {
        Self(0)
    }

    pub fn init(&mut self) {
        *self = Self::new();
    }

    pub fn validate_as_level3_descriptor(&mut self) {
        self.0 |= 0b11;
    }

    pub fn validate_as_table_descriptor(&mut self) {
        self.0 |= 0b11;
    }

    pub fn validate_as_block_descriptor(&mut self) {
        self.0 |= 0b01
    }

    pub const fn is_validated(&self) -> bool {
        (self.0 & 0b11) != 0b00
    }

    pub const fn is_table_descriptor(&self) -> bool {
        (self.0 & 0b11) == 0b11
    }

    pub const fn is_block_descriptor(&self) -> bool {
        (self.0 & 0b11) == 0b01
    }

    pub const fn is_level3_descriptor(&self) -> bool {
        (self.0 & 0b11) == 0b11
    }

    pub const fn get_next_table_address(&self) -> usize {
        (self.0 & Self::TABLE_ADDRESS_MASK) as usize
    }

    pub fn set_output_address(&mut self, output_address: usize) {
        self.0 = (self.0 & !Self::OUTPUT_ADDRESS_MASK) | (output_address as u64) | Self::AF;
    }

    pub fn set_shareability(&mut self, shareability: Shareability) {
        self.0 = (self.0 & !Self::SH) | ((shareability as u64) << Self::SH_OFFSET);
    }

    pub fn set_permission(&mut self, permission: u64) {
        self.0 = (self.0 & !Self::S2AP) | (permission << Self::S2AP_OFFSET);
    }

    pub fn set_memory_attribute_write_back(&mut self) {
        self.0 = (self.0 & !Self::ATTR_INDEX) | Self::ATTR_WRITE_BACK;
    }
}

fn number_of_concatenated_page_tables(t0sz: u8, first_level: i8) -> usize {
    if t0sz > (43 - ((3 - first_level) as u8) * 9) {
        1
    } else {
        2usize.pow(((43 - ((3 - first_level) as u8) * 9) - t0sz) as u32)
    }
}

fn _map_address_stage2(
    physical_address: &mut usize,
    virtual_address: &mut usize,
    remaining_size: &mut usize,
    table_address: usize,
    permission: u64,
    table_level: i8,
    num_of_entries: usize,
) -> Result<(), ()> {
    let shift_level = 12 + 9 * (3 - table_level as usize);
    println!("table address: {:#X}", table_address);
    println!("shift_level: {shift_level}");
    let table_index = (*virtual_address >> shift_level) & (num_of_entries - 1);
    let table = unsafe {
        &mut *core::ptr::slice_from_raw_parts_mut(table_address as *mut TableEntry, num_of_entries)
    };

    if table_level == 3 {
        println!("mapping: {:#X} to {:#X}", virtual_address, physical_address);
        println!("level 3: {:#X}", table_address);
        for e in table[table_index..num_of_entries].iter_mut() {
            e.init();
            e.set_output_address(*physical_address);
            e.set_permission(permission);
            e.set_memory_attribute_write_back();
            e.set_shareability(Shareability::InterShareable);
            e.validate_as_level3_descriptor();
            *physical_address += PAGE_SIZE;
            *virtual_address += PAGE_SIZE;
            *remaining_size -= PAGE_SIZE;
            if *remaining_size == 0 {
                return Ok(());
            }
        }
        return Ok(());
    }
    for e in table[table_index..num_of_entries].iter_mut() {
        let block_size = 1usize << shift_level;
        let mask = block_size - 1;
        if table_level >= 2
            && *remaining_size >= block_size
            && (*physical_address & mask) == 0
            && (*virtual_address & mask) == 0
        {
            /* ブロックエントリ */
            println!(
                "block entry: {:#X} ~ {:#X} => {:#X} ~ {:#X}",
                *virtual_address,
                *virtual_address as u64 + block_size as u64,
                *physical_address,
                *physical_address as u64 + block_size as u64
            );
            e.init();
            e.set_output_address(*physical_address);
            e.set_permission(permission);
            e.set_memory_attribute_write_back();
            e.set_shareability(Shareability::InterShareable);
            e.validate_as_block_descriptor();
            *physical_address += block_size;
            *virtual_address += block_size;
            *remaining_size -= block_size;
            if *remaining_size == 0 {
                return Ok(());
            }
            continue;
        }
        let mut next_table_address = e.get_next_table_address();
        if !e.is_table_descriptor() {
            next_table_address = allocate_memory(1, Some(12))?;
            println!("next table address: {:#X}", next_table_address);
            for n in unsafe {
                &mut *core::ptr::slice_from_raw_parts_mut(
                    next_table_address as *mut TableEntry,
                    512,
                )
            } {
                n.init();
            }
            e.init();
            e.set_output_address(next_table_address);
            e.validate_as_table_descriptor();
        }
        println!("table index: {:#X}\nlevel: {:#X}", table_index, table_level);
        println!("num_of_entries: {:#X}", num_of_entries);
        println!("next table {:#X}", next_table_address);
        _map_address_stage2(
            physical_address,
            virtual_address,
            remaining_size,
            next_table_address,
            permission,
            table_level + 1,
            512,
        )?;
        if *remaining_size == 0 {
            return Ok(());
        }
    }
    return Ok(());
}

pub fn map_address_stage2(
    mut physical_address: usize,
    mut virtual_address: usize,
    mut map_size: usize,
    is_readable: bool,
    is_writable: bool,
) -> Result<(), ()> {
    if (map_size & ((1usize << PAGE_SHIFT) - 1)) != 0 {
        println!("Map size is not aligned.");
        return Err(());
    }
    let page_table_address = (get_vttbr_el2() & VTTBR_BADDR) as usize;
    let vtcr_el2 = get_vtcr_el2();
    let sl0 = ((vtcr_el2 & VTCR_EL2_SL0) >> VTCR_EL2_SL0_BITS_OFFSET) as u8;
    let t0sz = ((vtcr_el2 & VTCR_EL2_T0SZ) >> VTCR_EL2_T0SZ_BITS_OFFSET) as u8;
    let table_level: i8 = match sl0 {
        0b00 => 2,
        0b01 => 1,
        0b10 => 0,
        0b11 => 3,
        _ => unreachable!(),
    };
    println!("get table level {:#X}", table_level);
    println!(
        "first num_of_entries: {:#X}",
        number_of_concatenated_page_tables(t0sz, table_level) * 512
    );
    _map_address_stage2(
        &mut physical_address,
        &mut virtual_address,
        &mut map_size,
        page_table_address,
        ((is_writable as u64) << 1) | (is_readable as u64),
        table_level,
        number_of_concatenated_page_tables(t0sz, table_level) * 512,
    )?;
    flush_tlb_el1();
    Ok(())
}

pub fn setup_stage_2_translation() -> Result<(), ()> {
    let ps = get_id_aa64mmfr0_el1() & ID_AA64MMFR0_EL1_PARANGE;
    let (t0sz, table_level) = match ps {
        0b000 => (32u64, 1i8),
        0b001 => (28u64, 1i8),
        0b010 => (24u64, 1i8),
        0b011 => (22u64, 0i8),
        0b100 => (20u64, 0i8),
        0b101 => (16u64, 0i8),
        _ => (16u64, 0i8),
    };
    let sl0 = if table_level == 1 { 0b01u64 } else { 0b10u64 };
    let number_of_tables = number_of_concatenated_page_tables(t0sz as u8, table_level);
    let table_address =
        allocate_page_table_for_stage_2(table_level, t0sz as u8, true, number_of_tables as u8)
            .unwrap();
    for e in unsafe {
        &mut *core::ptr::slice_from_raw_parts_mut(
            table_address as *mut TableEntry,
            number_of_tables * 512,
        )
    } {
        e.init();
    }

    let vtcr_el2: u64 = VTCR_EL2_RES1
        | (ps << VTCR_EL2_PS_BITS_OFFSET)
        | (0 << VTCR_EL2_TG0_BITS_OFFSET)
        | (0b11 << VTCR_EL2_SH0_BITS_OFFSET)
        | (0b11 << VTCR_EL2_ORG0_BITS_OFFSET)
        | (0b11 << VTCR_EL2_IRG0_BITS_OFFSET)
        | (sl0 << VTCR_EL2_SL0_BITS_OFFSET)
        | (t0sz << VTCR_EL2_T0SZ_BITS_OFFSET);
    println!("set table address: {:#X}", table_address);
    unsafe {
        set_vtcr_el2(vtcr_el2);
        set_vttbr_el2(table_address as u64);
    }
    Ok(())
}

/// Allocate page table for stage 2 with suitable address alignment
#[inline(always)]
fn allocate_page_table_for_stage_2(
    look_up_level: i8,
    t0sz: u8,
    is_for_ttbr: bool,
    number_of_tables: u8,
) -> Result<usize, ()> {
    assert_ne!(number_of_tables, 0);
    let alignment = if is_for_ttbr {
        ((64 - ((PAGE_SHIFT - 3) as usize * (4 - look_up_level) as usize) - t0sz as usize).max(4))
            .min(12)
            + (number_of_tables as usize - 1)
    } else {
        assert_eq!(number_of_tables, 1);
        STAGE_2_PAGE_SHIFT
    };
    match allocate_memory(number_of_tables as usize, Some(alignment)) {
        Ok(address) => Ok(address),
        Err(err) => {
            println!("Failed to allocate the page table: {:?}", err);
            Err(())
        }
    }
}
