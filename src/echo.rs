use std::io::Write;

use serde::{Deserialize, Serialize};

type NodeId = String;
type MsgId = usize;

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
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum Response {
    InitOk,
    EchoOk { echo: String },
    GenerateOk { id: u64 },
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

fn init() -> NodeId {
    let msg = read_message();
    if let MessageBody::Request {
        msg_id,
        request: Request::Init { node_id, .. },
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
        node_id
    } else {
        panic!("expected Init message");
    }
}

impl Into<Response> for Request {
    fn into(self) -> Response {
        match self {
            Request::Init { .. } => panic!("unexpected Init request"),
            Request::Echo { echo } => Response::EchoOk { echo },
            Request::Generate => Response::GenerateOk { id: 42 },
        }
    }
}

fn main() {
    let self_id = init();
    loop {
        let msg = read_message();
        if msg.dest != self_id {
            continue;
        }
        match msg.body {
            MessageBody::Request { request, msg_id } => {
                let response = Message {
                    src: msg.dest,
                    dest: msg.src,
                    body: MessageBody::Response {
                        response: request.into(),
                        in_reply_to: msg_id,
                    },
                };
                write_message(&response);
            }
            MessageBody::Response { .. } => panic!("unexpected Response message"),
        }
    }
}
