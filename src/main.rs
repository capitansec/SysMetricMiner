use sysinfo::{System, SystemExt, Processor, ProcessorExt};
use serde::{Serialize, Deserialize};
use serde_json::to_string;
use lapin::{Connection, ConnectionProperties, options::*, types::FieldTable, BasicProperties, message::DeliveryResult};
use tokio::runtime::Runtime;
use tokio::time::{sleep, Duration};
use tokio::time::{timeout};

use dotenv::dotenv;
use std::env;

#[derive(Serialize, Deserialize)]
struct SystemInfo {
    total_cpu: f64,
    used_cpu: f64,
    used_memory: u64,
    total_memory: u64,
    timestamp: u64,
}

fn calculate_cpu_amount(processors: &[Processor]) -> (f64, f64) {
    let total_cpu: f64 = processors.iter().map(|p| f64::from(p.get_cpu_speed())).sum();
    let used_cpu: f64 = processors.iter().map(|p| f64::from(p.get_cpu_usage())).sum();

    (total_cpu, used_cpu)
}

fn get_system_info() -> SystemInfo {
    let mut system = System::new_all();
    system.refresh_all();

    let total_memory = system.get_total_memory();
    let used_memory = system.get_used_memory();
    let processors = system.get_processors();
    let (total_cpu, used_cpu) = calculate_cpu_amount(&processors);

    SystemInfo {
        total_cpu,
        used_cpu,
        used_memory,
        total_memory,
        timestamp: chrono::Utc::now().timestamp() as u64,
    }
}

async fn send_to_rabbitmq(message: String) {
    let addr = env::var("AMQP_ADDR").expect("Environment variable AMQP_ADDR must be set");

    let timeout_duration = Duration::from_secs(10);

    match timeout(timeout_duration, Connection::connect(&addr, ConnectionProperties::default())).await {
        Ok(Ok(conn)) => {
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
        Ok(Err(err)) => {
            eprintln!("Failed to connect to RabbitMQ: {}", err);
        }
        Err(_) => {
            eprintln!("Timed out while connecting to RabbitMQ");
        }
    }
}
fn main() {
    dotenv().ok();
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
