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

struct Node {
    /// The id of this node
    id: NodeId,

    /// Local counter for the next message id to use
    last_msg_id: MsgId,

    /// Neighboring nodes in the cluster (including this node) and which values we know that they have seen
    cluster: HashMap<NodeId, HashSet<MsgValue>>,

    outbound_requests: HashMap<MsgId, Request>,
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

    fn broadcast(&mut self) {
        let mut last_msg_id = self.last_msg_id;
        let all_values = self.cluster[&self.id].iter().cloned().collect::<HashSet<_>>();
        for (dest, seen_values) in self.cluster.iter_mut() {
            let unseen_values = all_values.difference(seen_values);

            for value in unseen_values {
                let msg_id = last_msg_id + 1;
                last_msg_id += 1;

                let request = Request::Broadcast { value: *value };
                let msg = Message {
                    src: self.id.clone(),
                    dest: dest.clone(),
                    body: MessageBody::Request {
                        request: request.clone(),
                        msg_id: last_msg_id + 1,
                    },
                };
                self.outbound_requests.insert(msg_id, request);
                msg.send();
            }
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
                    Request::Broadcast { value } => {
                        // We know that we have seen this value
                        self.cluster
                            .get_mut(&self.id)
                            .expect("own id is in cluster")
                            .insert(*value);

                        // We know that the sender has seen this value
                        self.cluster
                            .get_mut(&msg.src)
                            .map(|seen_values| seen_values.insert(*value));

                        msg.respond_with(Response::BroadcastOk);
                        self.broadcast();
                    }
                    Request::Read => msg.respond_with(Response::ReadOk {
                        values: self.cluster[&self.id].clone(),
                    }),
                    Request::Topology { topology } => {
                        for id in topology.get(&self.id).expect("our own id is in topology") {
                            if !self.cluster.contains_key(id) {
                                self.cluster.insert(id.clone(), HashSet::new());
                            }
                        }
                        msg.respond_with(Response::TopologyOk);
                    }
                },
                MessageBody::Response { response, in_reply_to } => match response {
                    Response::BroadcastOk => {
                        match self.outbound_requests.remove(&in_reply_to) {
                            Some(Request::Broadcast { value }) => {
                                // We know that the destination has seen this value if they replied with BroadcastOk
                                self.cluster
                                    .get_mut(&msg.src)
                                    .expect("destination id in cluster")
                                    .insert(value);
                            }
                            Some(req) => panic!("unexpected request {:?}", req),
                            None => {}
                        }
                    }
                    _ => panic!("unexpected response {:?}", response),
                },
            }
        }
    }
}

fn main() {
    Node::init().run();
}
