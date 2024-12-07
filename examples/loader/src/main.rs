#![cfg_attr(feature = "axstd", no_std)]
#![cfg_attr(feature = "axstd", no_main)]
#![feature(asm_const)]

#[cfg(feature = "axstd")]
use axstd as std;

use std::{
    println,
    process::exit,
};

const PLASH_START: usize = 0xffff_ffc0_2200_0000;

const MAX_APP_SIZE: usize = 0x80000;

const RUN_START: usize = 0xffff_ffc0_8010_0000;

use core::{
    mem::size_of,
    slice::{
        from_raw_parts,
        from_raw_parts_mut,
    }
};

#[derive(Clone, Copy)]
struct ImageHeader {
    app_num: u32,
}

#[derive(Clone, Copy)]
struct AppHeader {
    app_size: usize,
}

const SYS_HELLO: usize = 1;
const SYS_PUTCHAR: usize = 2;
const SYS_EXIT: usize = 3;

static mut ABI_TABLE: [usize; 16] = [0; 16];

fn register_abi(num: usize, handle: usize) {
    unsafe { ABI_TABLE[num] = handle; }
}

fn abi_hello() {
    println!("[ABI:Hello] Hello, Apps!");
}

fn abi_putchar(c: char) {
    const LEGACY_CONSOLE_PUTCHAR: usize = 1;
    unsafe {
        core::arch::asm!(
            "ecall",
            in("a7") LEGACY_CONSOLE_PUTCHAR,
            in("a0") c as usize,
        );
    }
}

fn abi_exit() -> ! {
    println!("[ABI:Exit] Exit!");
    exit(0);
}

#[cfg_attr(feature = "axstd", no_mangle)]
fn main() {

    println!("Load payload ...");

    let image_header = unsafe {*(PLASH_START as *const ImageHeader)};
    let app_num = image_header.app_num;
    println!("app_num: {}", app_num);

    let mut offset_of_image = PLASH_START + size_of::<ImageHeader>();
    let mut offset_of_exec_zone = RUN_START;
    for _i in 0..app_num {
        let app_header = unsafe {*(offset_of_image as *const AppHeader)};
        let app_size = app_header.app_size;
        println!("app_size: {}", app_size);

        offset_of_image += size_of::<AppHeader>();
        let app_in_image = unsafe {from_raw_parts(offset_of_image as *const u8, app_size.min(MAX_APP_SIZE))};

        let run_code = unsafe {from_raw_parts_mut(offset_of_exec_zone as *mut u8, app_size.min(MAX_APP_SIZE))};
        run_code.copy_from_slice(app_in_image);
        println!("code {:?}; address [{:?}]", run_code, run_code.as_ptr());

        offset_of_image += app_size;
        offset_of_exec_zone += app_size;
    }
    println!("Load payload ok!");

    register_abi(SYS_HELLO, abi_hello as usize);
    register_abi(SYS_PUTCHAR, abi_putchar as usize);
    register_abi(SYS_EXIT, abi_exit as usize);

    println!("Execute app ...");

    // execute app
    unsafe { core::arch::asm!("
        la      a7, {abi_table}
        li      t2, {run_start}
        jalr    t2
        j       .",
        run_start = const RUN_START+0x1010,
        abi_table = sym ABI_TABLE,
    )}

}