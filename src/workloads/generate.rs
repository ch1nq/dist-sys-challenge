use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::node::NodeId;
use crate::workloads::workload::Workload;
use crate::{message, node};

pub struct GenerateWorkload;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged, rename_all = "snake_case")]
pub enum Request {
    Generate,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged, rename_all = "snake_case")]
pub enum Response {
    GenerateOk { id: Uuid },
}

impl Workload for GenerateWorkload {
    type Request = Request;
    type Response = Response;

    fn new(_id: &NodeId) -> Self {
        GenerateWorkload
    }

    fn handle_request(
        &mut self,
        _request: &Self::Request,
        _msg_id: message::MsgId,
        _src: &node::NodeId,
    ) -> Self::Response {
        Response::GenerateOk { id: Uuid::new_v4() }
    }

    fn handle_response(&mut self, _response: &Self::Response, _in_reply_to: message::MsgId, _src: &node::NodeId) {}
}
