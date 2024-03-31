use std::collections::HashSet;
use std::sync::mpsc::{channel, Sender};
use std::thread;

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
    pub(crate) workload_1: P1,
    pub(crate) workload_2: P2,
}

impl<P1: Workload + 'static, P2: Workload + 'static> Workload for MultiWorkload<P1, P2> {
    type Request = Either<P1::Request, P2::Request>;
    type Response = Either<P1::Response, P2::Response>;

    fn new(id: NodeId, all_nodes: HashSet<NodeId>, tx: Sender<Body<Self>>) -> Self {
        let (send_a, recv_a) = channel();
        let tx_a = tx.clone();
        thread::spawn(move || {
            for body in recv_a {
                let wrapped_body = match body {
                    Body::Request { dest, request } => Body::Request {
                        dest,
                        request: Either::A(request),
                    },
                    Body::Response {
                        dest,
                        in_reply_to,
                        response,
                    } => Body::Response {
                        dest,
                        in_reply_to,
                        response: Either::A(response),
                    },
                };
                tx_a.send(wrapped_body).expect("send failed");
            }
        });

        let (send_b, recv_b) = channel();
        let tx_b = tx;
        thread::spawn(move || {
            for body in recv_b {
                let wrapped_body = match body {
                    Body::Request { dest, request } => Body::Request {
                        dest,
                        request: Either::B(request),
                    },
                    Body::Response {
                        dest,
                        in_reply_to,
                        response,
                    } => Body::Response {
                        dest,
                        in_reply_to,
                        response: Either::B(response),
                    },
                };
                tx_b.send(wrapped_body).expect("send failed");
            }
        });

        MultiWorkload {
            workload_1: P1::new(id.clone(), all_nodes.clone(), send_a),
            workload_2: P2::new(id, all_nodes, send_b),
        }
    }

    fn handle_request(
        &mut self,
        request: &Self::Request,
        src: &NodeId,
        reponse_factory: impl FnOnce(Self::Response) -> Body<Self>,
    ) {
        match request {
            Either::A(req) => self
                .workload_1
                .handle_request(req, src, |res| match reponse_factory(Either::A(res)) {
                    Body::Request {
                        dest,
                        request: Either::A(request),
                    } => Body::Request { dest, request },
                    Body::Response {
                        dest,
                        in_reply_to,
                        response: Either::A(response),
                    } => Body::Response {
                        dest,
                        in_reply_to,
                        response,
                    },
                    _ => panic!("Workload A sent a reponse type of workload B"),
                }),
            Either::B(req) => self
                .workload_2
                .handle_request(req, src, |res| match reponse_factory(Either::B(res)) {
                    Body::Request {
                        dest,
                        request: Either::B(request),
                    } => Body::Request { dest, request },
                    Body::Response {
                        dest,
                        in_reply_to,
                        response: Either::B(response),
                    } => Body::Response {
                        dest,
                        in_reply_to,
                        response,
                    },
                    _ => panic!("Workload B sent a reponse type of workload A"),
                }),
        }
    }

    fn handle_response(&mut self, response: &Self::Response, in_reply_to: MsgId, src: &NodeId) {
        match response {
            Either::A(res) => self.workload_1.handle_response(res, in_reply_to, src),
            Either::B(res) => self.workload_2.handle_response(res, in_reply_to, src),
        }
    }
}
