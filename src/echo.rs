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
        message: MsgValue,
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

fn read_message() -> Message {
    let stdin = std::io::stdin();
    let mut deserializer = serde_json::Deserializer::from_reader(stdin.lock());
    Message::deserialize(&mut deserializer).expect("failed to read message")
}

fn write_message(msg: &Message) {
    let mut handle = std::io::stdout().lock();
    serde_json::to_writer(&mut handle, msg).expect("failed to write message");
    handle.write(b"\n").expect("failed to write newline");
}

struct Node {
    id: NodeId,
    cluster: HashSet<NodeId>,
    seen: HashSet<MsgValue>,
}

impl Node {
    fn init() -> Self {
        let msg = read_message();
        if let MessageBody::Request {
            msg_id,
            request: Request::Init { node_id, node_ids },
        } = msg.body
        {
            let response = Message {
                src: msg.dest,
                dest: msg.src,
                body: MessageBody::Response {
                    response: Response::InitOk,
                    in_reply_to: msg_id,
                },
            };
            write_message(&response);
            Node {
                id: node_id,
                cluster: node_ids.into_iter().collect(),
                seen: HashSet::new(),
            }
        } else {
            panic!("expected Init message");
        }
    }

    fn handle_request(&mut self, request: Request) -> Option<Response> {
        match request {
            Request::Init { .. } => panic!("unexpected Init request"),
            Request::Echo { echo } => Some(Response::EchoOk { echo }),
            Request::Generate => Some(Response::GenerateOk { id: Uuid::new_v4() }),
            Request::Broadcast { message } => {
                self.seen.insert(message);
                Some(Response::BroadcastOk)
            }
            Request::Read => Some(Response::ReadOk {
                messages: self.seen.clone(),
            }),
            Request::Topology { topology } => {
                self.cluster = topology
                    .get(&self.id)
                    .expect("node not in topology")
                    .clone();
                Some(Response::TopologyOk)
            }
        }
    }

    fn handle_messages(mut self) -> ! {
        loop {
            let msg = read_message();
            if msg.dest != self.id {
                continue;
            }
            match msg.body {
                MessageBody::Request { request, msg_id } => {
                    if let Some(response) = self.handle_request(request) {
                        let body = MessageBody::Response {
                            response,
                            in_reply_to: msg_id,
                        };
                        let response = Message {
                            src: msg.dest,
                            dest: msg.src,
                            body,
                        };
                        write_message(&response);
                    }
                }
                MessageBody::Response { .. } => panic!("unexpected Response message"),
            }
        }
    }
}

fn main() {
    Node::init().handle_messages();
}
