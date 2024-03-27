use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::{message::MsgId, node::NodeId, workloads::workload::Workload};

use super::workload;

pub struct InitWorkload;

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

    fn new(_id: &NodeId) -> Self {
        InitWorkload
    }

    fn handle_request(
        &mut self,
        _request: &Self::Request,
        _msg_id: MsgId,
        _src: &NodeId,
    ) -> impl IntoIterator<Item = workload::Body<Self>> {
        vec![workload::Body::Response(Response::InitOk)]
    }

    fn handle_response(&mut self, _response: &Self::Response, _in_reply_to: MsgId, _src: &NodeId) {
        panic!("InitProtocol does not handle responses")
    }
}
