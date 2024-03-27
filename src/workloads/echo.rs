use serde::{Deserialize, Serialize};

use crate::{message::MsgId, node::NodeId, workloads::workload};

pub struct EchoWorkload;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Echo {
    echo: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type", rename = "echo_ok")]
pub struct EchoOk {
    echo: String,
}

impl workload::Workload for EchoWorkload {
    type Request = Echo;
    type Response = EchoOk;

    fn new(_id: &NodeId) -> Self {
        EchoWorkload
    }

    fn handle_request(
        &mut self,
        request: &Self::Request,
        _msg_id: MsgId,
        _src: &NodeId,
    ) -> impl IntoIterator<Item = workload::Body<Self>> {
        vec![workload::Body::Response(EchoOk {
            echo: request.echo.clone(),
        })]
    }

    fn handle_response(&mut self, _response: &EchoOk, _in_reply_to: MsgId, _src: &NodeId) {}
}
