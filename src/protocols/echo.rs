use serde::{Deserialize, Serialize};

use crate::{
    message::MsgId,
    node::{NodeFields, NodeId},
    protocols::protocol::NodeProtocol,
};

pub struct EchoProtocol;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Echo {
    echo: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type", rename = "echo_ok")]
pub struct EchoOk {
    echo: String,
}

impl NodeProtocol for EchoProtocol {
    type Request = Echo;
    type Response = EchoOk;

    fn new() -> Self {
        EchoProtocol
    }

    fn handle_request(
        &mut self,
        _node: &mut NodeFields,
        request: &Self::Request,
        _msg_id: MsgId,
        _src: &NodeId,
    ) -> Self::Response {
        EchoOk {
            echo: request.echo.clone(),
        }
    }

    fn handle_response(&mut self, _node: &mut NodeFields, _response: &EchoOk, _src: NodeId, _in_reply_to: MsgId) {}
}
