use std::{collections::HashSet, sync::mpsc::Sender};

use serde::{Deserialize, Serialize};

use crate::{message::MsgId, node::NodeId, workloads::workload::Workload};

use super::workload::Body;

pub struct InitWorkload {
    tx: Sender<Body<Self>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Init {
    pub node_id: NodeId,
    pub node_ids: HashSet<NodeId>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Response {
    InitOk,
}

impl Workload for InitWorkload {
    type Request = Init;
    type Response = Response;

    fn new(_id: &NodeId, tx: Sender<Body<Self>>) -> Self {
        InitWorkload { tx }
    }

    fn handle_request(
        &mut self,
        _request: &Self::Request,
        _src: &NodeId,
        reponse_factory: impl FnOnce(Self::Response) -> Body<Self>,
    ) {
        self.tx.send(reponse_factory(Response::InitOk)).expect("send failed");
    }

    fn handle_response(&mut self, _response: &Self::Response, _in_reply_to: MsgId, _src: &NodeId) {
        panic!("InitProtocol does not handle responses")
    }
}
