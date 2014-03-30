/*
 * Copyright (c) 2014, David Renshaw (dwrenshaw@gmail.com)
 *
 * See the LICENSE file in the capnproto-rust root directory.
 */

use any::{AnyPointer};
use common::{MessageSize};
use layout::{DecodeResult, FromStructReader, FromStructBuilder, HasStructSize};
use message::{MallocMessageBuilder};
use serialize::{OwnedSpaceMessageReader};
use std;
use std::vec::Vec;

pub struct ResultFuture<Results, Pipeline> {
    answer_port : std::comm::Receiver<~OwnedSpaceMessageReader>,
    answer_result : Option<~OwnedSpaceMessageReader>,
    pipeline : Pipeline,
}

pub trait RequestHook {
    fn message<'a>(&'a mut self) -> &'a mut MallocMessageBuilder;
    fn send(~self) -> ResultFuture<AnyPointer::Reader, AnyPointer::Pipeline>;
}

pub struct Request<Params, Results, Pipeline> {
    hook : ~RequestHook
}

impl <Params, Results, Pipeline > Request <Params, Results, Pipeline> {
    pub fn new(hook : ~RequestHook) -> Request <Params, Results, Pipeline> {
        Request { hook : hook }
    }
}
impl <Params, Results, Pipeline : FromTypelessPipeline> Request <Params, Results, Pipeline> {
    pub fn send(self) -> ResultFuture<Results, Pipeline> {
        let ResultFuture {answer_port, answer_result, pipeline} = self.hook.send();
        ResultFuture { answer_port : answer_port, answer_result : answer_result,
                        pipeline : FromTypelessPipeline::new(pipeline) }
    }
}

pub trait FromClientHook {
    fn new(~ClientHook:Send) -> Self;
}

pub trait ClientHook : Send {
    fn copy(&self) -> ~ClientHook:Send;
    fn new_call(&self,
                interface_id : u64,
                method_id : u16,
                size_hint : Option<MessageSize>)
                -> Request<AnyPointer::Builder, AnyPointer::Reader, AnyPointer::Pipeline>;
    fn call(&self, interface_id : u64, method_id : u16, context : ~CallContextHook:Send);

    // HACK
    fn get_descriptor(&self) -> ~std::any::Any;
}

pub trait ServerHook {
    fn new_client(unused : Option<Self>, server : ~Server:Send) -> Client;
}

pub trait FromServer<T, U> {
    fn new(hook : Option<T>, server : ~U) -> Self;
}

pub struct Client {
    hook : ~ClientHook:Send
}

impl Client {
    pub fn new(hook : ~ClientHook:Send) -> Client {
        Client { hook : hook }
    }

    pub fn new_call<Params, Results, Pipeline>(&self,
                                               interface_id : u64,
                                               method_id : u16,
                                               size_hint : Option<MessageSize>)
                                               -> Request<Params, Results, Pipeline> {
        let typeless = self.hook.new_call(interface_id, method_id, size_hint);
        Request { hook : typeless.hook }
    }
}

pub struct CallContext<Params, Results> {
    hook : ~CallContextHook:Send,
}

impl <Params, Results> CallContext<Params, Results> {
    pub fn fail(self) {self.hook.fail();}
    pub fn done(self) {self.hook.done();}
}

impl <'a, Params : FromStructReader<'a>, Results : FromStructBuilder<'a> + HasStructSize>
CallContext<Params, Results> {
    pub fn get<'a>(&'a mut self) -> (Params, Results) {
        let (any_params, any_results) = self.hook.get();
        (any_params.get_as_struct().unwrap(), any_results.get_as_struct().unwrap())
    }
}

pub trait CallContextHook {
    fn get<'a>(&'a mut self) -> (AnyPointer::Reader<'a>, AnyPointer::Builder<'a>);
    fn fail(~self);
    fn done(~self);
}

pub trait Server {
    fn dispatch_call(&mut self, interface_id : u64, method_id : u16,
                     context : CallContext<AnyPointer::Reader, AnyPointer::Builder>) -> DecodeResult<()>;

}

// Where should this live?
pub fn internal_get_typed_context<Params, Results>(
    typeless : CallContext<AnyPointer::Reader, AnyPointer::Builder>)
    -> CallContext<Params, Results> {
    CallContext { hook : typeless.hook }
}


pub trait PipelineHook {
    fn copy(&self) -> ~PipelineHook;
    fn get_pipelined_cap(&self, ops : Vec<PipelineOp::Type>) -> ~ClientHook:Send;
}

pub mod PipelineOp {

    #[deriving(Clone)]
    pub enum Type {
        Noop,
        GetPointerField(u16),
    }
}

pub trait FromTypelessPipeline {
    fn new (typeless : AnyPointer::Pipeline) -> Self;
}
