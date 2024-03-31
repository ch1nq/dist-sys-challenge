use std::{collections::HashSet, sync::mpsc::Sender};

use serde::{Deserialize, Serialize};

use crate::{message::MsgId, node::NodeId, workloads::workload};

use super::workload::Body;

pub struct EchoWorkload {
    tx: Sender<Body<Self>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Echo {
    echo: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type", rename = "echo_ok")]
pub struct EchoOk {
    echo: String,
}

impl workload::Workload for EchoWorkload {
    type Request = Echo;
    type Response = EchoOk;

    fn new(_id: NodeId, _all_nodes: HashSet<NodeId>, tx: Sender<Body<Self>>) -> Self {
        EchoWorkload { tx }
    }

    fn handle_request(
        &mut self,
        request: &Self::Request,
        _src: &NodeId,
        reponse_factory: impl FnOnce(Self::Response) -> Body<Self>,
    ) {
        self.tx
            .send(reponse_factory(EchoOk {
                echo: request.echo.clone(),
            }))
            .expect("send failed");
    }

    fn handle_response(&mut self, _response: &EchoOk, _in_reply_to: MsgId, _src: &NodeId) {}
}
