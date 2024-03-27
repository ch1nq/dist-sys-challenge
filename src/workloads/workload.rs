use serde::{de::DeserializeOwned, Serialize};

use crate::{message::MsgId, node::NodeId};

pub enum Body<W: Workload + ?Sized> {
    Request(NodeId, W::Request),
    Response(W::Response),
}

pub trait Workload {
    type Request: DeserializeOwned + Serialize + Clone + std::fmt::Debug;
    type Response: DeserializeOwned + Serialize + Clone + std::fmt::Debug;

    fn new(id: &NodeId) -> Self;

    fn handle_request(
        &mut self,
        request: &Self::Request,
        msg_id: MsgId,
        src: &NodeId,
    ) -> impl IntoIterator<Item = Body<Self>>;
    fn handle_response(&mut self, response: &Self::Response, in_reply_to: MsgId, src: &NodeId);
}
