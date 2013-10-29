/*
 * Copyright (c) 2013, David Renshaw (dwrenshaw@gmail.com)
 *
 * See the LICENSE file in the capnproto-rust root directory.
 */

#[feature(globs)];
#[feature(macro_rules)];

#[link(name = "capnproto-rust-benchmark", vers = "alpha", author = "dwrensha")];

#[crate_type = "bin"];

extern mod capnprust;

pub mod common;

pub mod carsales_capnp;
pub mod carsales;

pub mod catrank_capnp;
pub mod catrank;

pub mod eval_capnp;
pub mod eval;

macro_rules! passByObject(
    ( $testcase:ident, $iters:expr ) => ({
            let mut rng = common::FastRand::new();
            for _ in range(0, $iters) {
                let messageReq = capnprust::message::MessageBuilder::new_default();
                let messageRes = capnprust::message::MessageBuilder::new_default();

                let request = messageReq.initRoot::<$testcase::RequestBuilder>();
                let response = messageRes.initRoot::<$testcase::ResponseBuilder>();
                let expected = $testcase::setupRequest(&mut rng, request);

                do request.asReader |requestReader| {
                    $testcase::handleRequest(requestReader, response);
                }

                do response.asReader |responseReader| {
                    if (! $testcase::checkResponse(responseReader, expected)) {
                        println("Incorrect response.");
                    }
                }

                messageReq.release();
                messageRes.release();
            }
        });
    )

macro_rules! passByBytes(
    ( $testcase:ident, $iters:expr ) => ({
            let mut rng = common::FastRand::new();
            for _ in range(0, $iters) {
                let messageReq = capnprust::message::MessageBuilder::new_default();
                let messageRes = capnprust::message::MessageBuilder::new_default();

                let request = messageReq.initRoot::<$testcase::RequestBuilder>();
                let _response = messageRes.initRoot::<$testcase::ResponseBuilder>();
                let _expected = $testcase::setupRequest(&mut rng, request);
                fail!("unimplemented");
            }
        });
    )

macro_rules! passByPipe(
    ( $testcase:ident, $iters:expr) => ({
            fail!("unimplemented");
        });
    )

macro_rules! doTestcase(
    ( $testcase:ident, $mode:expr, $reuse:expr, $compression:expr, $iters:expr ) => ({
            match $mode {
                ~"object" => passByObject!($testcase, $iters),
                ~"bytes" => passByBytes!($testcase, $iters),
                s => fail!("unrecognized mode: {}", s)
            }
        });
    )


pub fn main () {

    let args = std::os::args();

    if (args.len() != 6) {
        println!("USAGE: {} CASE MODE REUSE COMPRESSION ITERATION_COUNT", args[0]);
        return;
    }

    let iters = match from_str::<u64>(args[5]) {
        Some (n) => n,
        None => {
            println!("Could not parse a u64 from: {}", args[5]);
            return;
        }
    };

    match args[1] {
        ~"carsales" => doTestcase!(carsales, args[2], args[3], args[4], iters),
        ~"catrank" => doTestcase!(catrank, args[2], args[3], args[4], iters),
        ~"eval" => doTestcase!(eval, args[2], args[3], args[4], iters),
        s => fail!("unrecognized test case: {}", s)
    }
}