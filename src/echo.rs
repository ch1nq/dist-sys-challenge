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
#[serde(tag = "type", rename_all = "snake_case")]
enum Request {
    Init { node_id: NodeId, node_ids: Vec<NodeId> },
    Echo { echo: String },
    Generate,
    Broadcast { message: MsgValue },
    Topology { topology: HashMap<NodeId, HashSet<NodeId>> },
    Read,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum Response {
    InitOk,
    EchoOk { echo: String },
    GenerateOk { id: Uuid },
    TopologyOk,
    ReadOk { messages: HashSet<MsgValue> },
    BroadcastOk,
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

struct Node {
    /// The id of this node
    id: NodeId,

    /// Local counter for the next message id to use
    last_msg_id: MsgId,

    /// Neighboring nodes in the cluster (including this node) and the messages they have seen
    cluster: HashMap<NodeId, HashSet<MsgValue>>,
}

impl Node {
    fn init() -> Self {
        let mut de = serde_json::Deserializer::from_reader(std::io::stdin());
        let msg = Message::deserialize(&mut de).expect("failed to parse message");
        match &msg.body {
            MessageBody::Request {
                request: Request::Init { node_id, node_ids },
                ..
            } => {
                let cluster = node_ids.clone().into_iter().map(|id| (id, HashSet::new())).collect();
                msg.respond_with(Response::InitOk);
                Node {
                    id: node_id.clone(),
                    last_msg_id: 0,
                    cluster,
                }
            }
            _ => {
                panic!("expected Init message");
            }
        }
    }

    fn broadcast(&mut self, message: MsgValue) {
        let mut last_msg_id = self.last_msg_id;
        for (dest, seen_messages) in self.cluster.iter_mut() {
            if seen_messages.contains(&message) {
                continue;
            }
            let msg = Message {
                src: self.id.clone(),
                dest: dest.clone(),
                body: MessageBody::Request {
                    request: Request::Broadcast { message },
                    msg_id: last_msg_id + 1,
                },
            };
            msg.send();
            seen_messages.insert(message);
            last_msg_id += 1;
        }
        self.last_msg_id = last_msg_id;
    }

    fn run(mut self) {
        let stdin = std::io::stdin();
        let deserializer = serde_json::Deserializer::from_reader(stdin);
        for msg in deserializer.into_iter::<Message>() {
            let msg = msg.expect("failed to parse message");
            if msg.dest != self.id {
                continue;
            }
            match msg.body {
                MessageBody::Request { ref request, .. } => match request {
                    Request::Init { .. } => panic!("unexpected Init request"),
                    Request::Echo { echo } => msg.respond_with(Response::EchoOk { echo: echo.to_string() }),
                    Request::Generate => msg.respond_with(Response::GenerateOk { id: Uuid::new_v4() }),
                    Request::Broadcast { message } => {
                        self.cluster
                            .get_mut(&self.id)
                            .expect("our own id not in cluster")
                            .insert(*message);
                        self.cluster
                            .get_mut(&msg.src)
                            .map(|seen_messages| seen_messages.insert(*message));
                        self.broadcast(*message);
                        msg.respond_with(Response::BroadcastOk);
                    }
                    Request::Read => msg.respond_with(Response::ReadOk {
                        messages: self.cluster[&self.id].clone(),
                    }),
                    Request::Topology { topology } => {
                        for id in topology.get(&self.id).expect("our own id not in topology") {
                            if !self.cluster.contains_key(id) {
                                self.cluster.insert(id.clone(), HashSet::new());
                            }
                        }
                        msg.respond_with(Response::TopologyOk);
                    }
                },
                MessageBody::Response { response, .. } => match response {
                    Response::BroadcastOk => {}
                    _ => panic!("unexpected response {:?}", response),
                },
            }
        }
    }
}

fn main() {
    Node::init().run();
}
