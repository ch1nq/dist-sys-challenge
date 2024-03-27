use std::io::Write;

use serde::Deserialize;

use crate::{
    message::{Message, MessageBody, MsgId},
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

    /// Local counter for the next message id to use
    pub next_msg_id: MsgId,

    protocol: W,
}

impl<W: Workload> Node<W> {
    fn request<T: Workload>(&mut self, dest: NodeId, request: T::Request) {
        let msg_id = self.next_msg_id;
        self.send::<T>(dest, MessageBody::Request { msg_id, request });
        self.next_msg_id += 1;
    }

    fn send<T: Workload>(&mut self, dest: NodeId, body: MessageBody<T::Request, T::Response>) {
        let msg = Message::<T> {
            src: self.id.clone(),
            dest,
            body,
        };
        let mut stdout = std::io::stdout().lock();
        serde_json::to_writer(&mut stdout, &msg).expect("write message");
        stdout.write_all(b"\n").expect("write newline");
    }

    fn repond_to<T: Workload>(&mut self, msg: &Message<T>, response: T::Response) {
        let MessageBody::Request { msg_id, .. } = msg.body else {
            panic!("expected Request")
        };
        let in_reply_to = msg_id.clone();
        self.send::<T>(msg.src.clone(), MessageBody::Response { in_reply_to, response });
    }
}

impl<W: Workload> Node<W> {
    pub fn init() -> Self {
        let mut de = serde_json::Deserializer::from_reader(std::io::stdin().lock());
        let msg = Message::<init::InitWorkload>::deserialize(&mut de).expect("a valid message");

        let MessageBody::Request { ref request, .. } = msg.body else {
            panic!("expected Request")
        };

        let mut node = Node {
            id: request.node_id.clone(),
            next_msg_id: 0,
            protocol: W::new(&request.node_id),
        };

        node.repond_to(&msg, init::Response::InitOk);
        node
    }

    fn handle_message(&mut self, message: Message<W>) {
        match message.body {
            MessageBody::Request { ref request, msg_id } => {
                let requests = self
                    .protocol
                    .handle_request(&request, msg_id, &message.src)
                    .into_iter()
                    .collect::<Vec<_>>();
                for request in requests {
                    match request {
                        Body::Request(node_id, request) => {
                            self.request::<W>(node_id, request);
                        }
                        Body::Response(response) => {
                            self.repond_to(&message, response);
                        }
                    }
                }
            }
            MessageBody::Response { response, in_reply_to } => {
                self.protocol.handle_response(&response, in_reply_to, &message.src);
            }
        }
    }

    pub fn run(mut self) {
        let deserializer = serde_json::Deserializer::from_reader(std::io::stdin());
        for msg in deserializer.into_iter::<Message<W>>() {
            let msg = msg.expect("a valid message");
            if msg.dest == self.id {
                self.handle_message(msg);
            }
        }
    }
}
