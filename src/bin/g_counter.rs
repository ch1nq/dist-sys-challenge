use dist_sys_challenge::{node, workloads::g_counter};

fn main() {
    node::Node::<g_counter::GCounterWorkload>::init().run();
}
