use dist_sys_challenge::{node, protocols::*};

// TODO: Generate using a macro
type P = multi::MultiProtocol<init::InitProtocol, broadcast::BroadcastProtocol>;

fn main() {
    node::Node::<P>::init().run();
}
