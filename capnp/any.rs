/*
 * Copyright (c) 2013-2014, David Renshaw (dwrenshaw@gmail.com)
 *
 * See the LICENSE file in the capnproto-rust root directory.
 */

pub mod AnyPointer {
    use std;
    use std::vec::Vec;

    use capability::{ClientHook, FromClientHook, PipelineHook, PipelineOp};
    use layout::{DecodeResult};
    use layout::{PointerReader, PointerBuilder, FromStructReader, FromStructBuilder,
                 HasStructSize, ToStructReader};
    use blob::{Text, Data};

    pub struct Reader<'a> {
        priv reader : PointerReader<'a>
    }

    impl <'a> Reader<'a> {
        #[inline]
        pub fn new<'b>(reader : PointerReader<'b>) -> Reader<'b> {
            Reader { reader : reader }
        }

        #[inline]
        pub fn is_null(&self) -> bool {
            self.reader.is_null()
        }

        #[inline]
        pub fn get_as_struct<T : FromStructReader<'a>>(&self) -> DecodeResult<T> {
            Ok(FromStructReader::new(try!(self.reader.get_struct(std::ptr::null()))))
        }

        pub fn get_as_text(&self) -> DecodeResult<Text::Reader<'a>> {
            self.reader.get_text(std::ptr::null(), 0)
        }

        pub fn get_as_data(&self) -> DecodeResult<Data::Reader<'a>> {
            self.reader.get_data(std::ptr::null(), 0)
        }

        pub fn get_as_capability<T : FromClientHook>(&self) -> T {
            FromClientHook::new(self.reader.get_capability())
        }


        //# Used by RPC system to implement pipelining. Applications
        //# generally shouldn't use this directly.
        pub fn get_pipelined_cap(&self, ops : &[PipelineOp::Type]) -> ~ClientHook:Send {
            let mut pointer = self.reader;

            for op in ops.iter() {
                match op {
                    &PipelineOp::Noop =>  { }
                    &PipelineOp::GetPointerField(idx) => {
                        pointer = pointer.get_struct(std::ptr::null()).unwrap().get_pointer_field(idx as uint)
                    }
                }
            }

            pointer.get_capability()
        }
    }

    pub struct Builder<'a> {
        priv builder : PointerBuilder<'a>
    }

    impl <'a> Builder<'a> {
        #[inline]
        pub fn new<'b>(builder : PointerBuilder<'a>) -> Builder<'a> {
            Builder { builder : builder }
        }

        pub fn get_as_struct<T : FromStructBuilder<'a> + HasStructSize>(&self) -> DecodeResult<T> {
            Ok(FromStructBuilder::new(
                try!(self.builder.get_struct(HasStructSize::struct_size(None::<T>), std::ptr::null()))))
        }

        pub fn init_as_struct<T : FromStructBuilder<'a> + HasStructSize>(&self) -> T {
            FromStructBuilder::new(
                self.builder.init_struct(
                    HasStructSize::struct_size(None::<T>)))
        }

        pub fn set_as_struct<T : ToStructReader<'a>>(&self, value : &T) {
            self.builder.set_struct(&value.struct_reader());
        }

        // XXX value should be a user client.
        pub fn set_as_capability(&self, value : ~ClientHook:Send) {
            self.builder.set_capability(value);
        }

        pub fn set_as_text(&self, value : &str) {
            self.builder.set_text(value);
        }

        pub fn set_as_data(&self, value : &[u8]) {
            self.builder.set_data(value);
        }

        #[inline]
        pub fn clear(&self) {
            self.builder.clear()
        }

        #[inline]
        pub fn as_reader(&self) -> Reader<'a> {
            Reader { reader : self.builder.as_reader() }
        }
    }

    pub struct Pipeline {
        hook : ~PipelineHook,
        ops : Vec<PipelineOp::Type>,
    }

    impl Pipeline {
        pub fn new(hook : ~PipelineHook) -> Pipeline {
            Pipeline { hook : hook, ops : Vec::new() }
        }

        pub fn noop(&self) -> Pipeline {
            Pipeline { hook : self.hook.copy(), ops : self.ops.clone() }
        }

        pub fn get_pointer_field(&self, pointer_index : u16) -> Pipeline {
            let mut new_ops = Vec::with_capacity(self.ops.len() + 1);
            for &op in self.ops.iter() {
                new_ops.push(op)
            }
            new_ops.push(PipelineOp::GetPointerField(pointer_index));
            Pipeline { hook : self.hook.copy(), ops : new_ops }
        }

        pub fn as_cap(&self) -> ~ClientHook:Send {
            self.hook.get_pipelined_cap(self.ops.clone())
        }
    }
}
