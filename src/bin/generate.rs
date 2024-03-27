use dist_sys_challenge::{node, protocols::generate};

fn main() {
    node::Node::<generate::GenerateProtocol>::init().run();
}
