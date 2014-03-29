/*
 * Copyright (c) 2013-2014, David Renshaw (dwrenshaw@gmail.com)
 *
 * See the LICENSE file in the capnproto-rust root directory.
 */

#![feature(globs)]
#![feature(macro_rules)]

#![crate_id="capnp"]
#![crate_type = "lib"]

pub mod any;
pub mod arena;
pub mod blob;
pub mod capability;
pub mod common;
pub mod endian;
pub mod io;
pub mod layout;
pub mod list;
pub mod mask;
pub mod message;
pub mod serialize;
pub mod serialize_packed;


#[cfg(test)]
pub mod layout_test;
#[cfg(test)]
pub mod serialize_packed_test;
