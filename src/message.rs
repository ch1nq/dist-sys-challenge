use serde::{Deserialize, Serialize};

use crate::{node::NodeId, workloads::workload::Workload};

pub type MsgId = usize;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct Message<P: Workload> {
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
