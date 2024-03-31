use rand::seq::IteratorRandom;
use serde::{Deserialize, Serialize};

use crate::workloads::workload::Workload;
use crate::{message, node::NodeId};
use rand::{self, thread_rng};
use std::collections::{HashMap, HashSet};
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::thread;

use super::workload::Body;

type CounterValue = usize;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Request {
    Add { delta: CounterValue },
    SyncState { state: HashMap<NodeId, CounterValue> },
    Read,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Response {
    AddOk,
    ReadOk { value: CounterValue },
}

pub struct GCounterWorkload {
    id: NodeId,
    tx: Sender<Body<Self>>,
    node_values: HashMap<NodeId, CounterValue>,
}

impl GCounterWorkload {
    fn sync(&self) {
        let mut rng = thread_rng();
        for dest in self
            .node_values
            .keys()
            .filter(|id| *id != &self.id)
            .choose_multiple(&mut rng, 4)
        {
            let request = Body::Request {
                dest: dest.clone(),
                request: Request::SyncState {
                    state: self.node_values.clone(),
                },
            };
            self.tx.send(request).expect("send failed");
        }
    }
}

impl Workload for GCounterWorkload {
    type Request = Request;
    type Response = Response;

    fn new(id: NodeId, all_nodes: HashSet<NodeId>, tx: Sender<Body<Self>>) -> Self {
        GCounterWorkload {
            id,
            tx,
            node_values: all_nodes.into_iter().map(|node_id| (node_id, 0)).collect(),
        }
    }

    fn handle_request(
        &mut self,
        request: &Self::Request,
        _src: &NodeId,
        reponse_factory: impl FnOnce(Self::Response) -> Body<Self>,
    ) {
        match request {
            Request::Add { delta } => {
                self.node_values.get_mut(&self.id).map(|value| *value += delta);
                self.sync();
                self.tx.send(reponse_factory(Response::AddOk)).expect("send failed");
            }
            Request::SyncState { state } => {
                let prev_value: CounterValue = self.node_values.values().sum();

                for (node_id, new_value) in state {
                    self.node_values
                        .entry(node_id.clone())
                        .and_modify(|old_value| {
                            if new_value > old_value {
                                *old_value = *new_value
                            }
                        })
                        .or_insert(*new_value);
                }

                let new_value: CounterValue = self.node_values.values().sum();
                if prev_value != new_value {
                    self.sync();
                }
            }
            Request::Read => {
                let value = self.node_values.values().sum();
                self.tx
                    .send(reponse_factory(Response::ReadOk { value }))
                    .expect("send failed");
            }
        }
    }

    fn handle_response(&mut self, response: &Response, _in_reply_to: message::MsgId, _src: &NodeId) {
        panic!("Did not expect response of type {:?}", response);
    }
}
