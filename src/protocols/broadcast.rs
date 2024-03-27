use serde::{Deserialize, Serialize};

use crate::protocols::protocol::NodeProtocol;
use crate::{message, node};
use std::collections::{HashMap, HashSet};

//////////////////////////////// BROADCAST PROTOCOL /////////////////////////////////
pub struct BroadcastProtocol {
    outbound_requests: HashMap<message::MsgId, OutboundRequest>,
    seen_values: HashMap<node::NodeId, HashSet<message::MsgValue>>,
}

#[derive(Debug)]
struct OutboundRequest {
    request: BroadcastRequest,
    dest: node::NodeId,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BroadcastRequest {
    value: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BroadcastOk;

impl NodeProtocol for BroadcastProtocol {
    type Request = BroadcastRequest;
    type Response = BroadcastOk;

    fn new() -> Self {
        BroadcastProtocol {
            outbound_requests: HashMap::new(),
            seen_values: HashMap::new(),
        }
    }

    fn handle_request(
        &mut self,
        node: &mut node::NodeFields,
        request: &Self::Request,
        msg_id: message::MsgId,
        src: &node::NodeId,
    ) -> Self::Response {
        BroadcastOk
    }

    fn handle_response(
        &mut self,
        node: &mut node::NodeFields,
        response: &Self::Response,
        src: node::NodeId,
        in_reply_to: message::MsgId,
    ) {
    }
}
