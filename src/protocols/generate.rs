use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::protocols::protocol::NodeProtocol;
use crate::{message, node};

pub struct GenerateProtocol;

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

impl NodeProtocol for GenerateProtocol {
    type Request = Request;
    type Response = Response;

    fn new() -> Self {
        GenerateProtocol
    }

    fn handle_request(
        &mut self,
        _node: &mut node::NodeFields,
        _request: &Self::Request,
        _msg_id: message::MsgId,
        _src: &node::NodeId,
    ) -> Self::Response {
        Response::GenerateOk { id: Uuid::new_v4() }
    }

    fn handle_response(
        &mut self,
        _node: &mut node::NodeFields,
        _response: &Self::Response,
        _src: node::NodeId,
        _in_reply_to: message::MsgId,
    ) {
    }
}
