use capnp;
use capnp::layout::DecodeResult;
use zmq;
use capnp_zmq;
use std;
use time;
use explorers_capnp::Grid;

enum OutputMode {
    Colors,
    Confidence
}

fn write_ppm(path : &std::path::Path, grid : Grid::Reader, mode : OutputMode) -> DecodeResult<()> {
    match std::io::File::open_mode(path, std::io::Truncate, std::io::Write) {
        Err(_e) => fail!("could not open"),
        Ok(writer) => {
            let mut buffered = std::io::BufferedWriter::new(writer);
            writeln!(&mut buffered, "P6");

            let cells = try!(grid.get_cells());
            let width = cells.size();
            assert!(width > 0);
            let height = try!(cells[0]).size();

            writeln!(&mut buffered, "{} {}", width, height);
            writeln!(&mut buffered, "255");

            for x in range(0, width) {
                assert!(try!(cells[x]).size() == height);
            }

            for y in range(0, height) {
                for x in range(0, width) {
                    let cell = try!(cells[x])[y];

                    match mode {
                        Colors => {
                            buffered.write_u8((cell.get_mean_red()).floor() as u8).unwrap();
                            buffered.write_u8((cell.get_mean_green()).floor() as u8).unwrap();
                            buffered.write_u8((cell.get_mean_blue()).floor() as u8).unwrap();
                        }
                        Confidence => {
                            let mut age = time::now().to_timespec().sec - cell.get_latest_timestamp();
                            if age < 0 { age = 0 };
                            age *= 25;
                            if age > 255 { age = 255 };
                            age = 255 - age;

                            let mut n = cell.get_number_of_updates();
                            n *= 10;
                            if n > 255 { n = 255 };

                            buffered.write_u8(0 as u8).unwrap();

                            buffered.write_u8(n as u8).unwrap();

                            buffered.write_u8(age as u8).unwrap();
                        }
                    }
                }
            }

            buffered.flush().unwrap();
        }
    };
    Ok(())
}

pub fn main() -> DecodeResult<()> {
    use capnp::message::MessageReader;

    let mut context = zmq::Context::new();
    let mut requester = context.socket(zmq::REQ).unwrap();

    assert!(requester.connect("tcp://localhost:5556").is_ok());

    let mut c : uint = 0;

    loop {
        requester.send([], 0).unwrap();

        let frames = capnp_zmq::recv(&mut requester).unwrap();
        let segments = capnp_zmq::frames_to_segments(frames);
        let reader = capnp::message::SegmentArrayMessageReader::new(segments,
                                                                    capnp::message::DefaultReaderOptions);
        let grid = try!(reader.get_root::<Grid::Reader>());

        println!("{}", grid.get_latest_timestamp());

        let filename = std::path::Path::new(format!("colors{:05}.ppm", c));
        try!(write_ppm(&filename, grid, Colors));

        let filename = std::path::Path::new(format!("conf{:05}.ppm", c));
        try!(write_ppm(&filename, grid, Confidence));

        c += 1;
        std::io::timer::sleep(5000);
    }
}
