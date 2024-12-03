#![cfg_attr(feature = "axstd", no_std)]
#![cfg_attr(feature = "axstd", no_main)]

#[cfg(feature = "axstd")]
use axstd::println;

const PLASH_START: usize = 0xffff_ffc0_2200_0000;

const MAX_APP_SIZE: usize = 0x100000;

use core::{
    mem::size_of,
    slice::from_raw_parts,
};

#[derive(Clone, Copy)]
struct ImageHeader {
    app_size: usize,
}

#[cfg_attr(feature = "axstd", no_mangle)]
fn main() {
    let pflash_start = PLASH_START as *const u8;

    println!("Load payload ...");

    let image_header = unsafe {*(pflash_start as *const ImageHeader)};

    let app_size = image_header.app_size;

    println!("App size: {}", app_size);

    let code = unsafe {
        from_raw_parts(pflash_start.add(size_of::<ImageHeader>()), app_size.max(MAX_APP_SIZE))
    };

    println!("content: {:?}: ", &code[..10.min(app_size)]);

    println!("Load payload ok!");
}