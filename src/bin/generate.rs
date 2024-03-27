use dist_sys_challenge::{node, workloads::generate};

fn main() {
    node::Node::<generate::GenerateWorkload>::init().run();
}
