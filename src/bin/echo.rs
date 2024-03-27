use dist_sys_challenge::{node, workloads::echo};

fn main() {
    node::Node::<echo::EchoWorkload>::init().run();
}
