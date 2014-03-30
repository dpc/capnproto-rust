#![crate_id="zmq-explorers"]
#![crate_type = "bin"]

extern crate zmq;
extern crate capnp;
extern crate rand;
extern crate time;

pub mod capnp_zmq;
pub mod explorers_capnp;
pub mod explorer;
pub mod collector;
pub mod viewer;


fn usage(s : &str) {
    println!("usage: {} [explorer|collector|viewer]", s);
    std::os::set_exit_status(1);
}

pub fn main() {

    let args = std::os::args();

    if args.len() < 2 {
        usage(args[0]);
        return;
    }

    match args[1].as_slice() {
        "explorer" => explorer::main(),
        "collector" => collector::main().unwrap(),
        "viewer" => viewer::main().unwrap(),
        _ => usage(args[0]),
    }

}
