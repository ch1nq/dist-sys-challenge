use serde::{Deserialize, Serialize};

use crate::{
    message::MsgId,
    node::{NodeFields, NodeId},
    protocols::protocol::NodeProtocol,
};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Either<A, B> {
    A(A),
    B(B),
}

pub struct MultiProtocol<P1, P2> {
    protocol_1: P1,
    protocol_2: P2,
}

impl<P1: NodeProtocol, P2: NodeProtocol> NodeProtocol for MultiProtocol<P1, P2> {
    type Request = Either<P1::Request, P2::Request>;
    type Response = Either<P1::Response, P2::Response>;

    fn new() -> Self {
        MultiProtocol {
            protocol_1: P1::new(),
            protocol_2: P2::new(),
        }
    }

    fn handle_request(
        &mut self,
        node: &mut NodeFields,
        request: &Self::Request,
        msg_id: MsgId,
        src: &NodeId,
    ) -> Self::Response {
        match request {
            Either::A(req) => Either::A(self.protocol_1.handle_request(node, req, msg_id, src)),
            Either::B(req) => Either::B(self.protocol_2.handle_request(node, req, msg_id, src)),
        }
    }

    fn handle_response(&mut self, node: &mut NodeFields, response: &Self::Response, src: NodeId, in_reply_to: MsgId) {
        match response {
            Either::A(res) => self.protocol_1.handle_response(node, res, src, in_reply_to),
            Either::B(res) => self.protocol_2.handle_response(node, res, src, in_reply_to),
        }
    }
}
