use serde::{Deserialize, Serialize};

use crate::workloads::workload::Workload;
use crate::{message, node::NodeId};
use std::collections::{HashMap, HashSet};

use super::workload;

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
    Gossip {
        #[serde(rename = "messages")]
        values: HashSet<MsgValue>,
    },
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
    GossipOk,
}

pub struct BroadcastWorkload {
    outbound_requests: HashMap<message::MsgId, OutboundRequest>,
    seen_values: HashMap<NodeId, HashSet<MsgValue>>,
    id: NodeId,
}

#[derive(Debug)]
struct OutboundRequest {
    value: HashSet<MsgValue>,
    dest: NodeId,
}

impl Workload for BroadcastWorkload {
    type Request = Request;
    type Response = Response;

    fn new(id: &NodeId) -> Self {
        BroadcastWorkload {
            outbound_requests: HashMap::new(),
            seen_values: HashMap::from([(id.clone(), HashSet::new())]),
            id: id.clone(),
        }
    }

    fn handle_request(
        &mut self,
        request: &Request,
        _msg_id: message::MsgId,
        src: &NodeId,
    ) -> impl IntoIterator<Item = workload::Body<Self>> {
        match request {
            Request::Topology { topology } => {
                self.seen_values
                    .extend(topology[&self.id].iter().cloned().map(|id| (id, HashSet::new())));
                vec![workload::Body::Response(Response::TopologyOk)]
            }
            Request::Broadcast { value } => {
                // We have now seen this value
                self.seen_values
                    .get_mut(&self.id)
                    .expect("own id should be in seen_values")
                    .insert(*value);

                // and we know that the sender has also seen this value
                self.seen_values
                    .get_mut(src)
                    .map(|seen_values| seen_values.insert(*value));

                vec![workload::Body::Response(Response::BroadcastOk)]
            }
            Request::Read => vec![workload::Body::Response(Response::ReadOk {
                values: self.seen_values[&self.id].clone(),
            })],
            Request::Gossip { values } => {
                self.seen_values
                    .get_mut(&self.id)
                    .expect("own id should be in seen_values")
                    .extend(values.into_iter());

                vec![workload::Body::Response(Response::GossipOk)]
            }
        }
    }

    fn handle_response(&mut self, response: &Response, in_reply_to: message::MsgId, _src: &NodeId) {
        match response {
            Response::BroadcastOk | Response::GossipOk => {
                if let Some(outbound_req) = self.outbound_requests.remove(&in_reply_to) {
                    self.seen_values
                        .get_mut(&outbound_req.dest)
                        .expect("src should be in cluster")
                        .extend(outbound_req.value);
                }
            }
            _ => panic!("Did not expect response of type {:?}", response),
        }
    }
}
