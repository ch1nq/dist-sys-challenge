use serde::{Deserialize, Serialize};

use crate::workloads::workload::Workload;
use crate::{message, node::NodeId};
use std::collections::{HashMap, HashSet};
use std::sync::mpsc::Sender;

use super::workload::Body;

type Key = String;
type MsgValue = usize;
type Offset = usize;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Request {
    /// Requests that a "msg" value be appended to a log identified by "key".
    Send { key: Key, msg: MsgValue },

    /// Requests that a node return messages from a set of logs starting from the given offset in each log
    Poll { offsets: HashMap<Key, Offset> },

    /// Informs the node that messages have been successfully processed up to and including the given offset
    CommitOffsets { offsets: HashMap<Key, Offset> },

    /// Requests a map of committed offsets for a given set of logs
    ListCommittedOffsets { keys: Vec<Key> },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Response {
    SendOk {
        offset: Offset,
    },
    PollOk {
        msgs: HashMap<Key, Vec<(Offset, MsgValue)>>,
    },
    CommitOffsetsOk,
    ListCommittedOffsetsOk {
        offsets: HashMap<Key, Offset>,
    },
}

#[derive(Default, Debug)]
struct Logs {
    commit_offset: Option<Offset>,
    entries: Vec<Option<MsgValue>>,
}

#[derive(Debug)]
pub struct KafkaWorkload {
    _id: NodeId,
    tx: Sender<Body<Self>>,
    logs: HashMap<Key, Logs>,
}

impl Workload for KafkaWorkload {
    type Request = Request;
    type Response = Response;

    fn new(id: NodeId, _all_nodes: HashSet<NodeId>, tx: Sender<Body<Self>>) -> Self {
        KafkaWorkload {
            _id: id,
            tx,
            logs: Default::default(),
        }
    }

    fn handle_request(
        &mut self,
        request: &Self::Request,
        _src: &NodeId,
        reponse_factory: impl FnOnce(Self::Response) -> Body<Self>,
    ) {
        match request {
            Request::Send { key, msg } => {
                let log_entries = &mut self.logs.entry(key.clone()).or_default().entries;
                let offset = log_entries.len();
                log_entries.push(Some(msg.clone()));
                self.tx
                    .send(reponse_factory(Response::SendOk { offset }))
                    .expect("send failed");
            }
            Request::Poll { offsets } => {
                let msgs = offsets
                    .iter()
                    .filter_map(|(key, offset)| {
                        self.logs
                            .get(key)
                            .and_then(|logs| logs.entries.get(*offset..))
                            .map(|entries| {
                                let key = key.clone();
                                let entries_filtered = entries
                                    .into_iter()
                                    .enumerate()
                                    .filter_map(|(i, msg)| msg.map(|msg| (offset + i, msg)))
                                    .collect::<Vec<_>>();
                                (key, entries_filtered)
                            })
                    })
                    .collect::<HashMap<_, _>>();
                self.tx
                    .send(reponse_factory(Response::PollOk { msgs }))
                    .expect("send failed");
            }
            Request::CommitOffsets { offsets } => {
                for (key, offset) in offsets {
                    let log = self.logs.entry(key.clone()).or_default();
                    log.commit_offset = Some(*offset);
                    if log.entries.len() < *offset {
                        log.entries.resize(*offset, None);
                    }
                }
                self.tx
                    .send(reponse_factory(Response::CommitOffsetsOk))
                    .expect("send failed");
            }
            Request::ListCommittedOffsets { keys } => {
                let offsets = keys
                    .iter()
                    .filter_map(|key| {
                        self.logs
                            .get(key)
                            .and_then(|logs| logs.commit_offset.map(|offset| (key.clone(), offset)))
                    })
                    .collect::<HashMap<_, _>>();
                self.tx
                    .send(reponse_factory(Response::ListCommittedOffsetsOk { offsets }))
                    .expect("send failed");
            }
        }
    }

    fn handle_response(&mut self, response: &Response, _in_reply_to: message::MsgId, _src: &NodeId) {
        panic!("Did not expect response of type {:?}", response);
    }
}
