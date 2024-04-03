use dist_sys_challenge::{node, workloads::kafka};

fn main() {
    node::Node::<kafka::KafkaWorkload>::init().run();
}
