#![no_std]
#![no_main]

use axlog::info;

mod abi;
use abi::{init_abis, ABI_TABLE};

mod load;
use load::load_elf;

#[no_mangle]
fn main() {
    init_abis();

    let entry = load_elf();

    info!("Execute app ...");
    unsafe {
        core::arch::asm!("
        la      a7, {abi_table}
        mv      t2, {entry}
        jalr    t2",
            entry = in(reg) entry,
            abi_table = sym ABI_TABLE,
        )
    }
}