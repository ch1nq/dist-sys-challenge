use dist_sys_challenge::{node, workloads::broadcast};

fn main() {
    node::Node::<broadcast::BroadcastWorkload>::init().run();
}
