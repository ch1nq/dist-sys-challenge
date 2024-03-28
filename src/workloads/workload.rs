use serde::{de::DeserializeOwned, Serialize};

use crate::{message::MsgId, node::NodeId};
use std::sync::mpsc::Sender;

pub enum Body<W: Workload + ?Sized> {
    Request {
        dest: NodeId,
        request: W::Request,
    },
    Response {
        dest: NodeId,
        in_reply_to: MsgId,
        response: W::Response,
    },
}

pub trait Workload {
    type Request: DeserializeOwned + Serialize + Clone + std::fmt::Debug + Send;
    type Response: DeserializeOwned + Serialize + Clone + std::fmt::Debug + Send;

    fn new(id: &NodeId, tx: Sender<Body<Self>>) -> Self;

    fn handle_request(
        &mut self,
        request: &Self::Request,
        src: &NodeId,
        reponse_factory: impl FnOnce(Self::Response) -> Body<Self>,
    );
    fn handle_response(&mut self, response: &Self::Response, in_reply_to: MsgId, src: &NodeId);
}
