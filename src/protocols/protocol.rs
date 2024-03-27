use serde::{de::DeserializeOwned, Serialize};

use crate::{
    message::MsgId,
    node::{NodeFields, NodeId},
};

pub trait NodeProtocol {
    type Request: DeserializeOwned + Serialize + Clone + std::fmt::Debug;
    type Response: DeserializeOwned + Serialize + Clone + std::fmt::Debug;

    fn new() -> Self;
    fn handle_request(
        &mut self,
        node: &mut NodeFields,
        request: &Self::Request,
        msg_id: MsgId,
        src: &NodeId,
    ) -> Self::Response;
    fn handle_response(&mut self, node: &mut NodeFields, response: &Self::Response, src: NodeId, in_reply_to: MsgId);
}
