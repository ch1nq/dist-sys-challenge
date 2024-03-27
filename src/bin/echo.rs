use dist_sys_challenge::{node, protocols::echo};

fn main() {
    node::Node::<echo::EchoProtocol>::init().run();
}
