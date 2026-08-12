//! Pure planning logic: ELF bytes in, flash erase/write plan out.
//!
//! Kept free of USB and filesystem access so the flash geometry rules are
//! testable on the host without a device.

use std::collections::BTreeMap;

use anyhow::{bail, Context, Result};
use object::read::elf::{ElfFile32, ProgramHeader};
use object::Endianness;

/// Start of the memory-mapped flash window on RP2040/RP2350.
pub const FLASH_START: u32 = 0x1000_0000;
/// One past the end of the flash window. Addresses at or above this belong to
/// RAM and are not flashed.
pub const FLASH_END: u32 = 0x2000_0000;
/// Erase granularity: the flash sector.
pub const SECTOR_SIZE: u32 = 4096;
/// Write granularity: the flash page.
pub const PAGE_SIZE: u32 = 256;
/// What a NOR flash cell reads as once erased.
///
/// Bytes inside a written page that no segment covers are filled with this, so
/// the page keeps the value it would have had if it were left alone.
pub const ERASED_BYTE: u8 = 0xFF;

/// A device-ready flash operation sequence.
///
/// Every `erase` entry is a `(address, length)` pair aligned to
/// [`SECTOR_SIZE`], and every `write` entry is a `(address, data)` pair whose
/// address is aligned to [`PAGE_SIZE`] and whose data length is a whole number
/// of pages, so both can be handed to PICOBOOT unmodified.
///
/// A linker packs segments back to back at arbitrary offsets, so a segment may
/// start part-way into a page. The plan is therefore built from a byte map
/// rather than from segment boundaries: the pages touched by any segment byte
/// are emitted whole, and the bytes in them that no segment covers are filled
/// with [`ERASED_BYTE`] to match freshly erased flash.
///
/// Consecutive pages are **coalesced** into one write chunk, and adjacent
/// erase sectors into one erase range, so neighbouring sectors yield a single
/// `(0x1000_0000, 8192)` entry rather than two. One range means one PICOBOOT
/// command. Both `erase` and `write` are sorted by ascending address.
#[derive(Debug, PartialEq, Eq)]
pub struct WritePlan {
    /// Sectors to erase, as `(address, length)`, sorted and coalesced.
    pub erase: Vec<(u32, u32)>,
    /// Page-aligned data to write, as `(address, data)`, sorted by address.
    pub write: Vec<(u32, Vec<u8>)>,
}

/// Builds the erase ranges and page-aligned writes for the flash-resident
/// PT_LOAD segments of `elf`.
///
/// Segments outside the flash window (RAM-resident code, for instance) and
/// segments with no file content are dropped; it is an error for none to
/// remain, and an error for two segments to claim the same byte address.
pub fn write_plan(elf: &[u8]) -> Result<WritePlan> {
    let bytes = byte_map(elf)?;
    let write = write_chunks(&bytes);
    let erase = erase_ranges(&write)?;

    Ok(WritePlan { erase, write })
}

/// Places every flash-resident segment byte at its absolute address.
///
/// A `BTreeMap` keeps the bytes ordered by address, which is what lets the
/// chunking walk them in one pass, and makes a double-claimed address —
/// two segments overlapping — directly detectable.
fn byte_map(elf: &[u8]) -> Result<BTreeMap<u32, u8>> {
    let file = ElfFile32::<Endianness>::parse(elf).context("input is not a 32-bit ELF file")?;
    let endian = file.endian();

    let mut bytes: BTreeMap<u32, u8> = BTreeMap::new();
    for header in file.elf_program_headers() {
        if header.p_type(endian) != object::elf::PT_LOAD {
            continue;
        }
        let data = header
            .data(endian, elf)
            .map_err(|()| anyhow::anyhow!("PT_LOAD segment data lies outside the ELF file"))?;
        if data.is_empty() {
            continue;
        }

        let base = header.p_paddr(endian);
        if !(FLASH_START..FLASH_END).contains(&base) {
            continue;
        }

        for (offset, byte) in data.iter().enumerate() {
            let offset = u32::try_from(offset)
                .with_context(|| format!("segment at {base:#010x} is too large for a u32 length"))?;
            let addr = base
                .checked_add(offset)
                .with_context(|| format!("segment at {base:#010x} wraps past the address space"))?;
            if bytes.insert(addr, *byte).is_some() {
                bail!(
                    "segments overlap at {addr:#010x}: two PT_LOAD segments claim the same \
                     flash byte, so the ELF cannot be flashed unambiguously"
                );
            }
        }
    }

    if bytes.is_empty() {
        bail!(
            "no loadable segment lies in the flash window \
             [{FLASH_START:#010x}, {FLASH_END:#010x}); nothing to flash"
        );
    }
    Ok(bytes)
}

/// Groups the mapped bytes into page-aligned, page-sized write chunks.
///
/// Every page holding at least one mapped byte is emitted in full; runs of
/// consecutive pages become a single chunk.
fn write_chunks(bytes: &BTreeMap<u32, u8>) -> Vec<(u32, Vec<u8>)> {
    let mut pages: Vec<u32> = bytes
        .keys()
        .map(|addr| addr - (addr % PAGE_SIZE))
        .collect();
    pages.dedup();

    let mut chunks: Vec<(u32, Vec<u8>)> = Vec::new();
    for page in pages {
        let contiguous = chunks
            .last()
            .is_some_and(|(start, data)| start + data.len() as u32 == page);
        if !contiguous {
            chunks.push((page, Vec::new()));
        }

        let (_, data) = chunks.last_mut().expect("a chunk was just ensured");
        data.extend((page..page + PAGE_SIZE).map(|addr| {
            bytes.get(&addr).copied().unwrap_or(ERASED_BYTE)
        }));
    }

    chunks
}

/// The minimal set of sectors covering `write`, coalesced and sorted.
fn erase_ranges(write: &[(u32, Vec<u8>)]) -> Result<Vec<(u32, u32)>> {
    let mut sectors: Vec<(u32, u32)> = write
        .iter()
        .map(|(addr, data)| {
            let len = u32::try_from(data.len())
                .with_context(|| format!("chunk at {addr:#010x} is too large for a u32 length"))?;
            let end = addr
                .checked_add(len)
                .with_context(|| format!("chunk at {addr:#010x} wraps past the address space"))?;
            let start = addr - (addr % SECTOR_SIZE);
            let end = end.next_multiple_of(SECTOR_SIZE).max(start + SECTOR_SIZE);
            Ok((start, end))
        })
        .collect::<Result<Vec<(u32, u32)>>>()?;
    sectors.sort_by_key(|(start, _)| *start);

    let mut coalesced: Vec<(u32, u32)> = Vec::new();
    for (start, end) in sectors {
        match coalesced.last_mut() {
            Some((_, prev_end)) if *prev_end >= start => *prev_end = (*prev_end).max(end),
            _ => coalesced.push((start, end)),
        }
    }

    Ok(coalesced
        .into_iter()
        .map(|(start, end)| (start, end - start))
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Builds a minimal 32-bit little-endian ELF carrying `segments` as
    /// PT_LOAD program headers, so the planner can be exercised without a
    /// linker.
    ///
    /// Each segment is given as `(paddr, data)`. Segment data is laid out back
    /// to back after the program header table.
    fn elf_with_segments(segments: &[(u32, &[u8])]) -> Vec<u8> {
        const EHDR_SIZE: usize = 52;
        const PHDR_SIZE: usize = 32;

        let phoff = EHDR_SIZE;
        let data_start = phoff + PHDR_SIZE * segments.len();

        let mut out = Vec::new();

        // ELF header: 32-bit, little-endian, ET_EXEC, EM_ARM.
        out.extend_from_slice(&[0x7f, b'E', b'L', b'F']);
        out.push(1); // EI_CLASS = ELFCLASS32
        out.push(1); // EI_DATA  = ELFDATA2LSB
        out.push(1); // EI_VERSION
        out.push(0); // EI_OSABI
        out.extend_from_slice(&[0; 8]); // EI_ABIVERSION + padding
        out.extend_from_slice(&2u16.to_le_bytes()); // e_type = ET_EXEC
        out.extend_from_slice(&40u16.to_le_bytes()); // e_machine = EM_ARM
        out.extend_from_slice(&1u32.to_le_bytes()); // e_version
        out.extend_from_slice(&0u32.to_le_bytes()); // e_entry
        out.extend_from_slice(&(phoff as u32).to_le_bytes()); // e_phoff
        out.extend_from_slice(&0u32.to_le_bytes()); // e_shoff
        out.extend_from_slice(&0u32.to_le_bytes()); // e_flags
        out.extend_from_slice(&(EHDR_SIZE as u16).to_le_bytes()); // e_ehsize
        out.extend_from_slice(&(PHDR_SIZE as u16).to_le_bytes()); // e_phentsize
        out.extend_from_slice(&(segments.len() as u16).to_le_bytes()); // e_phnum
        out.extend_from_slice(&0u16.to_le_bytes()); // e_shentsize
        out.extend_from_slice(&0u16.to_le_bytes()); // e_shnum
        out.extend_from_slice(&0u16.to_le_bytes()); // e_shstrndx
        assert_eq!(out.len(), EHDR_SIZE, "ELF header must be 52 bytes");

        // Program header table.
        let mut offset = data_start;
        for (paddr, data) in segments {
            out.extend_from_slice(&1u32.to_le_bytes()); // p_type = PT_LOAD
            out.extend_from_slice(&(offset as u32).to_le_bytes()); // p_offset
            out.extend_from_slice(&paddr.to_le_bytes()); // p_vaddr
            out.extend_from_slice(&paddr.to_le_bytes()); // p_paddr
            out.extend_from_slice(&(data.len() as u32).to_le_bytes()); // p_filesz
            out.extend_from_slice(&(data.len() as u32).to_le_bytes()); // p_memsz
            out.extend_from_slice(&5u32.to_le_bytes()); // p_flags = R+X
            out.extend_from_slice(&4u32.to_le_bytes()); // p_align
            offset += data.len();
        }
        assert_eq!(out.len(), data_start, "program headers must be contiguous");

        for (_, data) in segments {
            out.extend_from_slice(data);
        }
        out
    }

    #[test]
    fn sub_page_segment_fills_the_rest_of_the_page_with_erased_bytes() {
        let elf = elf_with_segments(&[(0x1000_0000, &[0xAB; 300])]);

        let plan = write_plan(&elf).expect("the segment lies in flash");

        assert_eq!(plan.erase, vec![(0x1000_0000, 4096)]);
        assert_eq!(plan.write.len(), 1);
        let (addr, data) = &plan.write[0];
        assert_eq!(*addr, 0x1000_0000);
        assert_eq!(data.len(), 512, "300 bytes span two 256-byte pages");
        assert_eq!(&data[..300], &[0xAB; 300]);
        assert_eq!(&data[300..], &[0xFF; 212], "gap bytes read as erased flash");
    }

    #[test]
    fn ram_segment_is_dropped() {
        let elf = elf_with_segments(&[
            (0x1000_0000, &[0x11; 256]),
            (0x2000_0000, &[0x22; 100]),
        ]);

        let plan = write_plan(&elf).expect("the flash segment survives");

        assert_eq!(plan.write.len(), 1, "only the flash segment is planned");
        let (addr, data) = &plan.write[0];
        assert_eq!(*addr, 0x1000_0000);
        assert_eq!(data.len(), 256);
        assert_eq!(&data[..], &[0x11; 256]);
    }

    #[test]
    fn page_aligned_segment_offset_into_a_sector_erases_the_whole_sector() {
        let elf = elf_with_segments(&[(0x1000_0100, &[0x5A; 256])]);

        let plan = write_plan(&elf).expect("the segment lies in flash");

        assert_eq!(plan.erase, vec![(0x1000_0000, 4096)]);
        assert_eq!(plan.write, vec![(0x1000_0100, vec![0x5A; 256])]);
    }

    #[test]
    fn misaligned_segment_is_padded_back_to_its_page() {
        let elf = elf_with_segments(&[(0x1000_01c0, &[0x11; 64])]);

        let plan = write_plan(&elf).expect("a misaligned start is legal");

        assert_eq!(plan.erase, vec![(0x1000_0000, 4096)]);
        assert_eq!(plan.write.len(), 1);
        let (addr, data) = &plan.write[0];
        assert_eq!(*addr, 0x1000_0100, "the write starts at the page boundary");
        assert_eq!(data.len(), 256);
        assert_eq!(&data[..192], &[0xFF; 192], "bytes before the segment stay erased");
        assert_eq!(&data[192..], &[0x11; 64]);
    }

    #[test]
    fn consecutive_pages_coalesce_into_one_chunk() {
        let elf = elf_with_segments(&[
            (0x1000_0000, &[0x01; 4096]),
            (0x1000_1000, &[0x02; 256]),
        ]);

        let plan = write_plan(&elf).expect("both segments lie in flash");

        assert_eq!(
            plan.erase,
            vec![(0x1000_0000, 8192)],
            "adjacent sectors coalesce into a single erase range"
        );
        assert_eq!(plan.write.len(), 1, "consecutive pages form one chunk");
        let (addr, data) = &plan.write[0];
        assert_eq!(*addr, 0x1000_0000);
        assert_eq!(data.len(), 4352);
        assert_eq!(&data[..4096], &[0x01; 4096]);
        assert_eq!(&data[4096..], &[0x02; 256]);
    }

    #[test]
    fn adjacent_segments_share_a_page() {
        let elf = elf_with_segments(&[
            (0x1000_0000, &[0xAA; 100]),
            (0x1000_0064, &[0xBB; 100]),
        ]);

        let plan = write_plan(&elf).expect("neither segment overlaps the other");

        assert_eq!(plan.write.len(), 1);
        let (addr, data) = &plan.write[0];
        assert_eq!(*addr, 0x1000_0000);
        assert_eq!(data.len(), 256);
        assert_eq!(&data[..100], &[0xAA; 100]);
        assert_eq!(&data[100..200], &[0xBB; 100]);
        assert_eq!(&data[200..], &[0xFF; 56]);
    }

    #[test]
    fn overlapping_segments_are_rejected() {
        let elf = elf_with_segments(&[
            (0x1000_0000, &[0xAA; 200]),
            (0x1000_0080, &[0xBB; 200]),
        ]);

        let err = write_plan(&elf).expect_err("overlapping segments must be rejected");

        let message = format!("{err:#}");
        assert!(
            message.contains("overlap"),
            "error should name the overlap, got: {message}"
        );
    }
}

