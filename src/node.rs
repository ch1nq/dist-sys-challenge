use serde::Deserialize;

use crate::{
    message::{Message, MessageBody, MsgId},
    workloads::{init, workload::Workload},
};

pub type NodeId = String;

#[derive(Debug)]
pub struct Node<P: Workload> {
    /// The id of this node
    pub id: NodeId,

    /// Local counter for the next message id to use
    pub last_msg_id: MsgId,

    protocol: P,
}

impl<W: Workload> Node<W> {
    pub fn init() -> Self {
        let mut de = serde_json::Deserializer::from_reader(std::io::stdin());
        let msg = Message::<init::InitWorkload>::deserialize(&mut de).expect("a valid message");

        let MessageBody::Request { ref request, msg_id } = msg.body else {
            panic!("expected Request")
        };

        let init_response = init::InitWorkload::new(&request.node_id).handle_request(request, msg_id, &msg.src);
        msg.respond_with(init_response);

        Node {
            id: request.node_id.clone(),
            last_msg_id: 0,
            protocol: W::new(&request.node_id),
        }
    }

    fn handle_message(&mut self, message: Message<W>) {
        match message.body {
            MessageBody::Request { ref request, msg_id } => {
                let response = self.protocol.handle_request(&request, msg_id, &message.src);
                message.respond_with(response);
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
