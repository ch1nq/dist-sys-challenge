use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::{
    message::MsgId,
    node::{NodeFields, NodeId},
    protocols::protocol::NodeProtocol,
};

pub struct InitProtocol;

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

impl NodeProtocol for InitProtocol {
    type Request = Init;
    type Response = Response;

    fn new() -> Self {
        panic!("InitProtocol does not need to be initialized")
    }

    fn handle_request(
        &mut self,
        _node: &mut NodeFields,
        _request: &Self::Request,
        _msg_id: MsgId,
        _src: &NodeId,
    ) -> Self::Response {
        panic!("InitProtocol does not handle requests")
    }

    fn handle_response(
        &mut self,
        _node: &mut NodeFields,
        _response: &Self::Response,
        _src: NodeId,
        _in_reply_to: MsgId,
    ) {
        panic!("InitProtocol does not handle responses")
    }
}
