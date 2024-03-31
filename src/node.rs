use serde::Deserialize;
use std::io::Write;
use std::sync::mpsc;
use std::thread;

use crate::{
    message::{Message, MessageBody},
    workloads::{
        init,
        workload::{Body, Workload},
    },
};

pub type NodeId = String;

#[derive(Debug)]
pub struct Node<W: Workload> {
    /// The id of this node
    pub id: NodeId,

    /// The workload this node is running
    workload: W,
}

fn send<T: Workload>(message: Message<T>) {
    let mut stdout = std::io::stdout().lock();
    serde_json::to_writer(&mut stdout, &message).expect("write message");
    stdout.write_all(b"\n").expect("write newline");
}

fn sender_thread<W: Workload + Send + 'static>(node_id: NodeId, outbox_recv: mpsc::Receiver<Body<W>>) {
    let mut next_msg_id = 0;
    for body in outbox_recv.into_iter() {
        match body {
            Body::Request { dest, request } => {
                let msg_id = next_msg_id;
                next_msg_id += 1;
                let msg = Message::<W> {
                    src: node_id.clone(),
                    dest,
                    body: MessageBody::Request { msg_id, request },
                };
                send::<W>(msg);
            }
            Body::Response {
                dest,
                in_reply_to,
                response,
            } => {
                let msg = Message::<W> {
                    src: node_id.clone(),
                    dest,
                    body: MessageBody::Response { in_reply_to, response },
                };
                send::<W>(msg);
            }
        };
    }
}

impl<W: Workload + Send + 'static> Node<W> {
    pub fn init() -> Self {
        let (outbox_send, outbox_recv) = mpsc::channel();

        let mut de = serde_json::Deserializer::from_reader(std::io::stdin().lock());
        let msg = Message::<init::InitWorkload>::deserialize(&mut de).expect("a valid message");

        let MessageBody::Request { ref request, msg_id } = msg.body else {
            panic!("expected Request")
        };

        let node_id = request.node_id.clone();
        thread::spawn(move || sender_thread(node_id.clone(), outbox_recv));

        let init_response = Message::<init::InitWorkload> {
            src: request.node_id.clone(),
            dest: msg.src.clone(),
            body: MessageBody::Response {
                in_reply_to: msg_id,
                response: init::Response::InitOk,
            },
        };
        send(init_response);

        Node {
            id: request.node_id.clone(),
            workload: W::new(request.node_id.clone(), request.node_ids.clone(), outbox_send),
        }
    }

    pub fn run(mut self) {
        let deserializer = serde_json::Deserializer::from_reader(std::io::stdin());
        for msg in deserializer.into_iter::<Message<W>>() {
            let msg = msg.expect("a valid message");
            if msg.dest != self.id {
                continue;
            };
            match msg.body {
                MessageBody::Request { ref request, msg_id } => {
                    let response_factory = |response| Body::Response {
                        dest: msg.src.clone(),
                        in_reply_to: msg_id,
                        response,
                    };
                    self.workload.handle_request(request, &msg.src, response_factory);
                }
                MessageBody::Response { response, in_reply_to } => {
                    self.workload.handle_response(&response, in_reply_to, &msg.src)
                }
            }
        }
    }
}
