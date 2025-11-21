use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::fmt::Write as _;
use std::io::Write;
use std::path::Path;
use std::process::Command;
use std::{env, path::PathBuf};

use stm32_metapac::metadata::{
    ALL_CHIPS, ALL_PERIPHERAL_VERSIONS, METADATA, MemoryRegion, MemoryRegionKind,
    PeripheralRccKernelClock, PeripheralRccRegister, PeripheralRegisters, StopMode,
};

fn mem_filter(chip: &str, region: &str) -> bool {
    // in STM32WB, SRAM2a/SRAM2b are reserved for the radio core.
    if chip.starts_with("STM32WB")
        && !chip.starts_with("STM32WBA")
        && !chip.starts_with("STM32WB0")
        && region.starts_with("SRAM2")
    {
        return false;
    }

    if region.starts_with("SDRAM_") || region.starts_with("FMC_") || region.starts_with("OCTOSPI_")
    {
        return false;
    }

    true
}

fn get_memory_range(memory: &[MemoryRegion], kind: MemoryRegionKind) -> (u32, u32, String) {
    let mut mems: Vec<_> = memory
        .iter()
        .filter(|m| m.kind == kind && m.size != 0)
        .collect();
    mems.sort_by_key(|m| m.address);

    let mut start = u32::MAX;
    let mut end = u32::MAX;
    let mut names = Vec::new();
    let mut best: Option<(u32, u32, String)> = None;
    for m in mems {
        if !mem_filter(&METADATA.name, &m.name) {
            continue;
        }

        if m.address != end {
            names = Vec::new();
            start = m.address;
            end = m.address;
        }

        end += m.size;
        names.push(m.name.to_string());

        if best.is_none() || end - start > best.as_ref().unwrap().1 {
            best = Some((start, end - start, names.join(" + ")));
        }
    }

    best.unwrap()
}

fn gen_memory_x(memory: &[MemoryRegion], out_dir: &Path) {
    let mut memory_x = String::new();

    let flash = get_memory_range(memory, MemoryRegionKind::Flash);
    let ram = get_memory_range(memory, MemoryRegionKind::Ram);

    write!(memory_x, "MEMORY\n{{\n").unwrap();
    writeln!(
        memory_x,
        "    FLASH : ORIGIN = 0x{:08x}, LENGTH = {:>4}K /* {} */",
        flash.0,
        flash.1 / 1024,
        flash.2
    )
    .unwrap();
    writeln!(
        memory_x,
        "    RAM   : ORIGIN = 0x{:08x}, LENGTH = {:>4}K /* {} */",
        ram.0,
        ram.1 / 1024,
        ram.2
    )
    .unwrap();
    write!(memory_x, "}}").unwrap();

    std::fs::write(out_dir.join("memory.x"), memory_x.as_bytes()).unwrap();
}

fn main() {
    let memory = {
        let single_bank_selected = env::var("CARGO_FEATURE_SINGLE_BANK").is_ok();
        let dual_bank_selected = env::var("CARGO_FEATURE_DUAL_BANK").is_ok();

        let single_bank_memory = METADATA.memory.iter().find(|mem| {
            mem.iter().any(|region| region.name.contains("BANK_1"))
                && !mem.iter().any(|region| region.name.contains("BANK_2"))
        });

        let dual_bank_memory = METADATA.memory.iter().find(|mem| {
            mem.iter().any(|region| region.name.contains("BANK_1"))
                && mem.iter().any(|region| region.name.contains("BANK_2"))
        });

        match (single_bank_selected, dual_bank_selected) {
            (true, true) => panic!("Both 'single-bank' and 'dual-bank' features enabled"),
            (true, false) => single_bank_memory
                .expect("The 'single-bank' feature is not supported on this dual bank chip"),
            (false, true) => dual_bank_memory
                .expect("The 'dual-bank' feature is not supported on this single bank chip"),
            (false, false) => {
                if METADATA.memory.len() != 1 {
                    panic!(
                        "Chip supports single and dual bank configuration. No Cargo feature to select one is enabled. Use the 'single-bank' or 'dual-bank' feature to make your selection"
                    )
                }
                METADATA.memory[0]
            }
        }
    };

    let out_dir = &PathBuf::from(env::var_os("OUT_DIR").unwrap());
    gen_memory_x(memory, out_dir);
    println!("cargo:rustc-link-search={}", out_dir.display());
}
