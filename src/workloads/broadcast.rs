use serde::{Deserialize, Serialize};

use crate::workloads::workload::Workload;
use crate::{message, node::NodeId};
use std::collections::{HashMap, HashSet};

type MsgValue = isize;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Request {
    Topology {
        topology: HashMap<NodeId, HashSet<NodeId>>,
    },
    Broadcast {
        #[serde(rename = "message")]
        value: MsgValue,
    },
    Read,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Response {
    TopologyOk,
    BroadcastOk,
    ReadOk {
        #[serde(rename = "messages")]
        values: HashSet<MsgValue>,
    },
}

pub struct BroadcastWorkload {
    outbound_broadcasts: HashMap<message::MsgId, OutboundBroadcastRequest>,
    seen_values: HashMap<NodeId, HashSet<MsgValue>>,
    id: NodeId,
}

#[derive(Debug)]
struct OutboundBroadcastRequest {
    value: MsgValue,
    dest: NodeId,
}

impl Workload for BroadcastWorkload {
    type Request = Request;
    type Response = Response;

    fn new(id: &NodeId) -> Self {
        BroadcastWorkload {
            outbound_broadcasts: HashMap::new(),
            seen_values: HashMap::from([(id.clone(), HashSet::new())]),
            id: id.clone(),
        }
    }

    fn handle_request(&mut self, request: &Request, _msg_id: message::MsgId, src: &NodeId) -> Self::Response {
        match request {
            Request::Topology { topology } => {
                self.seen_values
                    .extend(topology[&self.id].iter().cloned().map(|id| (id, HashSet::new())));
                Response::TopologyOk
            }
            Request::Broadcast { value } => {
                // We have now seen this value
                self.seen_values
                    .get_mut(&self.id)
                    .expect("own id should be in seen_values")
                    .insert(*value);

                // and we know that the sender has seen this value
                self.seen_values
                    .get_mut(src)
                    .map(|seen_values| seen_values.insert(*value));

                Response::BroadcastOk
            }
            Request::Read => Response::ReadOk {
                values: self.seen_values[&self.id].clone(),
            },
        }
    }

    fn handle_response(&mut self, response: &Response, in_reply_to: message::MsgId, _src: &NodeId) {
        match response {
            Response::BroadcastOk => {
                let outbound_broadcast = self
                    .outbound_broadcasts
                    .remove(&in_reply_to)
                    .expect("reponse should be a reply to a request we have sent");
                self.seen_values
                    .get_mut(&outbound_broadcast.dest)
                    .expect("src should be in cluster")
                    .insert(outbound_broadcast.value);
            }
            _ => panic!("Did not expect response of type {:?}", response),
        }
    }
}
