use serde::{Deserialize, Serialize};

use crate::{
    message::MsgId,
    node::NodeId,
    workloads::workload::{Body, Workload},
};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Either<A, B> {
    A(A),
    B(B),
}

pub struct MultiWorkload<P1, P2> {
    workload_1: P1,
    workload_2: P2,
}

impl<P1: Workload, P2: Workload> Workload for MultiWorkload<P1, P2> {
    type Request = Either<P1::Request, P2::Request>;
    type Response = Either<P1::Response, P2::Response>;

    fn new(id: &NodeId) -> Self {
        MultiWorkload {
            workload_1: P1::new(id),
            workload_2: P2::new(id),
        }
    }

    fn handle_request(
        &mut self,
        request: &Self::Request,
        msg_id: MsgId,
        src: &NodeId,
    ) -> impl IntoIterator<Item = Body<Self>> {
        match request {
            Either::A(req) => self
                .workload_1
                .handle_request(req, msg_id, src)
                .into_iter()
                .map(|body| match body {
                    Body::Request(node_id, req) => Body::Request(node_id, Either::A(req)),
                    Body::Response(res) => Body::Response(Either::A(res)),
                })
                .collect::<Vec<_>>(),
            Either::B(req) => self
                .workload_2
                .handle_request(req, msg_id, src)
                .into_iter()
                .map(|body| match body {
                    Body::Request(node_id, req) => Body::Request(node_id, Either::B(req)),
                    Body::Response(res) => Body::Response(Either::B(res)),
                })
                .collect::<Vec<_>>(),
        }
    }

    fn handle_response(&mut self, response: &Self::Response, in_reply_to: MsgId, src: &NodeId) {
        match response {
            Either::A(res) => self.workload_1.handle_response(res, in_reply_to, src),
            Either::B(res) => self.workload_2.handle_response(res, in_reply_to, src),
        }
    }
}
