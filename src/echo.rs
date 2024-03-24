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
enum MessageBody {
    Init {
        msg_id: MsgId,
        node_id: NodeId,
        node_ids: Vec<NodeId>,
    },
    InitOk {
        in_reply_to: MsgId,
    },
    Echo {
        msg_id: MsgId,
        echo: String,
    },
    EchoOk {
        msg_id: MsgId,
        in_reply_to: MsgId,
        echo: String,
    },
    Read {
        msg_id: MsgId,
        key: String,
    },
    ReadOk {
        msg_id: MsgId,
        in_reply_to: MsgId,
        value: String,
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
    match msg.body {
        MessageBody::Init {
            msg_id, node_id, ..
        } => {
            let response = Message {
                src: msg.dest,
                dest: msg.src,
                body: MessageBody::InitOk {
                    in_reply_to: msg_id,
                },
            };
            write_message(&response);
            node_id
        }
        _ => panic!("expected Init message"),
    }
}

fn main() {
    let node_id = init();
    loop {
        let msg = read_message();
        if msg.dest != node_id {
            panic!("unexpected destination: {}", msg.dest);
        }
        let response = match msg.body {
            MessageBody::Echo { msg_id, echo } => Message {
                src: msg.dest,
                dest: msg.src,
                body: MessageBody::EchoOk {
                    msg_id,
                    in_reply_to: msg_id,
                    echo,
                },
            },
            _ => panic!("unexpected message type"),
        };
        write_message(&response);
    }
}
