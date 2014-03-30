/*
 * Copyright (c) 2013-2014, David Renshaw (dwrenshaw@gmail.com)
 *
 * See the LICENSE file in the capnproto-rust root directory.
 */

#![feature(globs)]

#![crate_id="capnpc-rust"]
#![crate_type = "bin"]

extern crate collections;
extern crate capnp;

pub mod schema_capnp;
pub mod codegen;

pub fn main() {
    codegen::main().unwrap();
}
