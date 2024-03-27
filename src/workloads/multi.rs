use serde::{Deserialize, Serialize};

use crate::{message::MsgId, node::NodeId, workloads::workload::Workload};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Either<A, B> {
    A(A),
    B(B),
}

pub struct MultiWorkload<P1, P2> {
    protocol_1: P1,
    protocol_2: P2,
}

impl<P1: Workload, P2: Workload> Workload for MultiWorkload<P1, P2> {
    type Request = Either<P1::Request, P2::Request>;
    type Response = Either<P1::Response, P2::Response>;

    fn new(id: &NodeId) -> Self {
        MultiWorkload {
            protocol_1: P1::new(id),
            protocol_2: P2::new(id),
        }
    }

    fn handle_request(&mut self, request: &Self::Request, msg_id: MsgId, src: &NodeId) -> Self::Response {
        match request {
            Either::A(req) => Either::A(self.protocol_1.handle_request(req, msg_id, src)),
            Either::B(req) => Either::B(self.protocol_2.handle_request(req, msg_id, src)),
        }
    }

    fn handle_response(&mut self, response: &Self::Response, in_reply_to: MsgId, src: &NodeId) {
        match response {
            Either::A(res) => self.protocol_1.handle_response(res, in_reply_to, src),
            Either::B(res) => self.protocol_2.handle_response(res, in_reply_to, src),
        }
    }
}
