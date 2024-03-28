use std::sync::mpsc::Sender;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::node::NodeId;
use crate::workloads::workload::Workload;
use crate::{message, node};

use super::workload::Body;

pub struct GenerateWorkload {
    tx: Sender<Body<Self>>,
}

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

    fn new(_id: &NodeId, tx: Sender<Body<Self>>) -> Self {
        GenerateWorkload { tx }
    }

    fn handle_request(
        &mut self,
        _request: &Self::Request,
        _src: &NodeId,
        reponse_factory: impl FnOnce(Self::Response) -> Body<Self>,
    ) {
        self.tx
            .send(reponse_factory(Response::GenerateOk { id: Uuid::new_v4() }))
            .expect("send failed");
    }

    fn handle_response(&mut self, _response: &Self::Response, _in_reply_to: message::MsgId, _src: &node::NodeId) {}
}
