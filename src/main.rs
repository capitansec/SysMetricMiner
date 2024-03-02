use sysinfo::{System, SystemExt, Processor, ProcessorExt};
use serde::{Serialize, Deserialize};
use serde_json::to_string;
use lapin::{Connection, ConnectionProperties, options::*, types::FieldTable, BasicProperties, message::DeliveryResult};
use tokio::runtime::Runtime;
use tokio::time::{sleep, Duration};
use std::env;

#[derive(Serialize, Deserialize)]
struct SystemInfo {
    cpu_usage_percent: f64,
    used_memory: u64,
    total_memory: u64,
    timestamp: u64,
}

fn calculate_average_cpu_usage(processors: &[Processor]) -> f64 {
    let total_cpu_usage: f64 = processors.iter().map(|p| f64::from(p.get_cpu_usage())).sum();
    let count = processors.len() as f64;

    if count > 0.0 {
        total_cpu_usage / count
    } else {
        0.0
    }
}

fn get_system_info() -> SystemInfo {
    let mut system = System::new_all();
    system.refresh_all();

    let total_memory = system.get_total_memory();
    let used_memory = system.get_used_memory();
    let cpu_usage = system.get_processors();

    let cpu_usage_percent = calculate_average_cpu_usage(&cpu_usage);

    SystemInfo {
        cpu_usage_percent,
        used_memory,
        total_memory,
        timestamp: chrono::Utc::now().timestamp() as u64,
    }
}

async fn send_to_rabbitmq(message: String) {
    let addr = "amqp://SuperUser:SuperPassword@127.0.0.1:5672/%2f";
    let conn = Connection::connect(&addr, ConnectionProperties::default()).await.expect("Failed to connect to RabbitMQ");
    let channel = conn.create_channel().await.expect("Failed to create a channel");
    let queue = "system_info_queue";

    let _queue = channel.queue_declare(
        queue,
        QueueDeclareOptions::default(),
        FieldTable::default(),
    ).await.expect("Failed to declare a queue");

    let confirm = channel.basic_publish(
        "",
        queue,
        BasicPublishOptions::default(),
        message.as_bytes(),
        BasicProperties::default(),
    ).await.expect("Failed to publish message");

    println!("Sent: {}", message);
}

fn main() {
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        loop {
            let system_info = get_system_info();
            let message = to_string(&system_info).unwrap();
            send_to_rabbitmq(message).await;
            tokio::time::sleep(Duration::from_secs(3)).await;
        }
    });
}
