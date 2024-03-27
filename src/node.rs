use std::collections::HashSet;

use serde::Deserialize;

use crate::{
    message::{Message, MessageBody, MsgId},
    protocols::init,
    protocols::protocol::NodeProtocol,
};

pub type NodeId = String;
#[derive(Debug)]
pub struct NodeFields {
    /// The id of this node
    id: NodeId,

    /// Local counter for the next message id to use
    last_msg_id: MsgId,

    /// Neighboring nodes in the cluster (including this node) and which values we know that they have seen
    cluster: HashSet<NodeId>,
}

#[derive(Debug)]
pub struct Node<P: NodeProtocol> {
    fields: NodeFields,
    protocol: P,
}

impl<P: NodeProtocol> Node<P> {
    pub fn init() -> Self {
        let mut de = serde_json::Deserializer::from_reader(std::io::stdin());
        let msg = Message::<init::InitProtocol>::deserialize(&mut de).expect("a valid message");

        let MessageBody::Request { ref request, .. } = msg.body else {
            panic!("expected Request")
        };

        msg.respond_with(init::Response::InitOk);
        Node {
            fields: NodeFields {
                id: request.node_id.clone(),
                last_msg_id: 0,
                cluster: request.node_ids.clone(),
            },
            protocol: P::new(),
        }
    }

    fn handle_message(&mut self, message: Message<P>) {
        match message.body {
            MessageBody::Request { ref request, msg_id } => {
                let response = self
                    .protocol
                    .handle_request(&mut self.fields, &request, msg_id, &message.src);
                message.respond_with(response);
            }
            MessageBody::Response { response, in_reply_to } => {
                self.protocol
                    .handle_response(&mut self.fields, &response, message.src, in_reply_to);
            }
        }
    }

    pub fn run(mut self) {
        let deserializer = serde_json::Deserializer::from_reader(std::io::stdin());
        for msg in deserializer.into_iter::<Message<P>>() {
            let msg = msg.expect("a valid message");
            if msg.dest == self.fields.id {
                self.handle_message(msg);
            }
        }
    }
}
