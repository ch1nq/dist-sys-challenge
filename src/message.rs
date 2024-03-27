use std::io::Write;

use serde::{Deserialize, Serialize};

use crate::{node::NodeId, protocols::protocol::NodeProtocol};

pub type MsgId = usize;
pub type MsgValue = isize;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct Message<P: NodeProtocol> {
    /// A string identifying the node this message came from
    pub src: NodeId,

    /// A string identifying the node this message is to
    pub dest: NodeId,

    /// The payload of the message
    pub body: MessageBody<P::Request, P::Response>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MessageBody<Request, Response> {
    Request {
        msg_id: MsgId,
        #[serde(flatten)]
        request: Request,
    },
    Response {
        in_reply_to: MsgId,
        #[serde(flatten)]
        response: Response,
    },
}

impl<S: NodeProtocol> Message<S> {
    pub fn send(self) {
        let mut handle = std::io::stdout().lock();
        serde_json::to_writer(&mut handle, &self).expect("failed to write message");
        handle.write(b"\n").expect("failed to write newline");
    }

    pub fn respond_with(&self, response: S::Response) {
        let MessageBody::Request { msg_id, .. } = &self.body else {
            panic!("expected Request")
        };
        let response_msg = Message::<S> {
            src: self.dest.clone(),
            dest: self.src.clone(),
            body: MessageBody::Response {
                in_reply_to: msg_id.clone(),
                response,
            },
        };
        response_msg.send();
    }
}
