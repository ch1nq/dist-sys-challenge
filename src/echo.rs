use rand::seq::IteratorRandom;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::io::Write;
use uuid::Uuid;

type NodeId = String;
type MsgId = usize;
type MsgValue = isize;

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Message {
    /// A string identifying the node this message came from
    src: NodeId,

    /// A string identifying the node this message is to
    dest: NodeId,

    /// The payload of the message
    body: MessageBody,
}

impl Message {
    fn send(self) {
        let mut handle = std::io::stdout().lock();
        serde_json::to_writer(&mut handle, &self).expect("failed to write message");
        handle.write(b"\n").expect("failed to write newline");
    }

    fn respond_with(&self, response: Response) {
        match &self.body {
            MessageBody::Request { msg_id, .. } => {
                let response_msg = Message {
                    src: self.dest.clone(),
                    dest: self.src.clone(),
                    body: MessageBody::Response {
                        in_reply_to: msg_id.clone(),
                        response,
                    },
                };
                response_msg.send();
            }
            _ => panic!("expected Request"),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum MessageBody {
    Request {
        #[serde(flatten)]
        request: Request,
        msg_id: MsgId,
    },
    Response {
        #[serde(flatten)]
        response: Response,
        in_reply_to: MsgId,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum Request {
    Init {
        node_id: NodeId,
        node_ids: Vec<NodeId>,
    },
    Echo {
        echo: String,
    },
    Generate,
    Broadcast {
        #[serde(rename = "message")]
        value: MsgValue,
    },
    Topology {
        topology: HashMap<NodeId, HashSet<NodeId>>,
    },
    Read,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum Response {
    InitOk,
    EchoOk {
        echo: String,
    },
    GenerateOk {
        id: Uuid,
    },
    TopologyOk,
    ReadOk {
        #[serde(rename = "messages")]
        values: HashSet<MsgValue>,
    },
    BroadcastOk,
}

#[derive(Debug)]
struct Node {
    /// The id of this node
    id: NodeId,

    /// Local counter for the next message id to use
    last_msg_id: MsgId,

    /// Neighboring nodes in the cluster (including this node) and which values we know that they have seen
    cluster: HashMap<NodeId, HashSet<MsgValue>>,

    /// We only have one outstanding request to each node at a time
    outbound_requests: HashMap<MsgId, OutboundRequest>,
}

#[derive(Debug)]
struct OutboundRequest {
    request: Request,
    dest: NodeId,
}

impl Node {
    fn init() -> Self {
        let mut de = serde_json::Deserializer::from_reader(std::io::stdin());
        let msg = Message::deserialize(&mut de).expect("failed to parse message");

        let MessageBody::Request {
            request: Request::Init { node_id, node_ids },
            ..
        } = &msg.body
        else {
            panic!("expected Init message");
        };

        let cluster = node_ids.clone().into_iter().map(|id| (id, HashSet::new())).collect();
        msg.respond_with(Response::InitOk);
        Node {
            id: node_id.clone(),
            last_msg_id: 0,
            cluster,
            outbound_requests: HashMap::new(),
        }
    }

    fn missing_values(&self) -> HashMap<NodeId, HashSet<MsgValue>> {
        let all_values = self.cluster[&self.id].iter().cloned().collect::<HashSet<_>>();
        self.cluster
            .iter()
            .map(|(id, seen_values)| {
                let unseen_values = all_values.difference(seen_values);
                (id.clone(), unseen_values.cloned().collect())
            })
            .collect()
    }

    fn broadcast(&mut self) {
        let mut last_msg_id = self.last_msg_id;
        let all_values = self.cluster[&self.id].iter().cloned().collect::<HashSet<_>>();
        let mut rng = rand::thread_rng();
        for (dest, seen_values) in self.cluster.iter_mut() {
            let unseen_values = all_values.difference(seen_values);

            // Pick a random unseen value to broadcast
            let Some(value) = unseen_values.into_iter().choose(&mut rng) else {
                continue;
            };

            let msg_id = last_msg_id + 1;
            last_msg_id += 1;

            let request = Request::Broadcast { value: *value };
            let msg = Message {
                src: self.id.clone(),
                dest: dest.clone(),
                body: MessageBody::Request {
                    request: request.clone(),
                    msg_id,
                },
            };
            msg.send();
            self.outbound_requests.insert(
                msg_id,
                OutboundRequest {
                    request,
                    dest: dest.clone(),
                },
            );
        }
        self.last_msg_id = last_msg_id;
    }

    fn handle_request(&mut self, request: &Request, _msg_id: MsgId, src: &NodeId) -> Response {
        match request {
            Request::Init { .. } => panic!("unexpected Init request"),
            Request::Echo { echo } => Response::EchoOk { echo: echo.clone() },
            Request::Generate => Response::GenerateOk { id: Uuid::new_v4() },
            Request::Broadcast { value } => {
                // We know that we have seen this value
                let our_seen_values = self.cluster.get_mut(&self.id).expect("own id is in cluster");
                our_seen_values.insert(*value);
                let our_seen_values_len = our_seen_values.len();

                // We know that the sender has seen this value
                if let Some(seen_values) = self.cluster.get_mut(src) {
                    seen_values.insert(*value);
                }

                // if self.outbound_requests.len() < self.cluster.len() * our_seen_values_len {
                //     self.broadcast();
                // }
                Response::BroadcastOk
            }
            Request::Read => Response::ReadOk {
                values: self.cluster[&self.id].clone(),
            },
            Request::Topology { topology } => {
                for id in topology.get(&self.id).expect("our own id is in topology") {
                    if !self.cluster.contains_key(id) {
                        self.cluster.insert(id.clone(), HashSet::new());
                    }
                }
                Response::TopologyOk
            }
        }
    }

    fn handle_response(&mut self, response: Response, src: NodeId, in_reply_to: MsgId) {
        if let Some(OutboundRequest {
            request: outbound_req, ..
        }) = self.outbound_requests.remove(&in_reply_to)
        {
            match (&outbound_req, &response) {
                (Request::Broadcast { value }, Response::BroadcastOk) => {
                    // We know that the destination has seen this value if they replied with BroadcastOk
                    self.cluster
                        .get_mut(&src)
                        .expect("destination id in cluster")
                        .insert(*value);
                }
                _ => panic!("Request and Response do not match: {:?} {:?}", outbound_req, response),
            }
        }
    }

    fn run(mut self) {
        let deserializer = serde_json::Deserializer::from_reader(std::io::stdin().lock());
        for msg in deserializer.into_iter::<Message>() {
            let msg = msg.expect("a valid message");
            if msg.dest != self.id {
                continue;
            }
            let src = msg.src.clone();
            match msg.body {
                MessageBody::Request { ref request, msg_id } => {
                    let response = self.handle_request(request, msg_id, &src);

                    msg.respond_with(response);

                    // We assume that the sender has not seen our outbound requests,
                    // so we can give up on them when we receive a response
                    self.outbound_requests
                        .retain(|_, OutboundRequest { dest, .. }| dest.clone() != src);
                }
                MessageBody::Response { response, in_reply_to } => self.handle_response(response, msg.src, in_reply_to),
            }
        }
        let num_seen_values = self.cluster.get(&self.id).expect("own id is in cluster").len();
        if (self.outbound_requests.len() as f64) < (self.cluster.len() as f64) * (num_seen_values as f64).sqrt() {
            self.broadcast();
        }
    }
}

fn main() {
    Node::init().run();
}
