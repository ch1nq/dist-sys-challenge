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
}

struct BroadcastState {
    tx: Sender<Body<BroadcastWorkload>>,
    seen_values: HashSet<MsgValue>,
    to_broadcast: HashSet<MsgValue>,
    neighbors: HashSet<NodeId>,
    all_nodes: HashSet<NodeId>,
}

pub struct BroadcastWorkload {
    id: NodeId,
    state: Arc<Mutex<BroadcastState>>,
}

impl BroadcastState {
    fn gossip(&mut self, rng: &mut rand::rngs::ThreadRng) {
        let values: HashSet<_> = self.to_broadcast.union(&self.seen_values).cloned().collect();

        for dest in self.all_nodes.iter().choose_multiple(rng, 4) {
            let request = Body::Request {
                dest: dest.clone(),
                request: Request::Gossip { values: values.clone() },
            };
            self.tx.send(request).expect("send failed");
        }

        self.seen_values.extend(self.to_broadcast.clone().into_iter());
        self.to_broadcast.clear();
    }
}

impl Workload for BroadcastWorkload {
    type Request = Request;
    type Response = Response;

    fn new(id: &NodeId, tx: Sender<Body<Self>>) -> Self {
        let state = Arc::new(Mutex::new(BroadcastState {
            tx,
            seen_values: Default::default(),
            to_broadcast: Default::default(),
            neighbors: Default::default(),
            all_nodes: Default::default(),
        }));

        let state_gossip = state.clone();
        thread::spawn(move || {
            let mut rng = rand::thread_rng();
            loop {
                thread::sleep(std::time::Duration::from_millis(500));
                state_gossip.lock().unwrap().gossip(&mut rng);
            }
        });

        BroadcastWorkload { id: id.clone(), state }
    }

    fn handle_request(
        &mut self,
        request: &Self::Request,
        _src: &NodeId,
        reponse_factory: impl FnOnce(Self::Response) -> Body<Self>,
    ) {
        let mut state = self.state.lock().unwrap();
        match request {
            Request::Topology { topology } => {
                state.neighbors.extend(topology[&self.id].clone().into_iter());
                state.all_nodes.extend(topology.keys().cloned());
                state
                    .tx
                    .send(reponse_factory(Response::TopologyOk))
                    .expect("send failed");
            }
            Request::Broadcast { value } => {
                // Only broadcast if we haven't seen this value before
                if !state.seen_values.contains(value) {
                    state.seen_values.insert(*value);
                    state.gossip(&mut thread_rng());
                }

                state
                    .tx
                    .send(reponse_factory(Response::BroadcastOk))
                    .expect("send failed");
            }
            Request::Read => {
                state
                    .tx
                    .send(reponse_factory(Response::ReadOk {
                        values: state.seen_values.clone(),
                    }))
                    .expect("send failed");
            }
            Request::Gossip { values } => {
                let unseen_values = values - &state.seen_values;
                state.to_broadcast.extend(unseen_values.into_iter());
            }
        }
    }

    fn handle_response(&mut self, response: &Response, _in_reply_to: message::MsgId, _src: &NodeId) {
        panic!("Did not expect response of type {:?}", response);
    }
}
