use dashmap::DashMap;
use std::sync::Arc;
use std::time::Duration;

#[derive(Debug, Clone)]
struct MyValue(i32);

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let dash = Arc::new(DashMap::new());
    dash.insert("cur", MyValue(0));
    dash.insert("cur2", MyValue(0));

    let handle_1 = dash.clone();
    let handle_2 = dash.clone();

    let loop_1 = tokio::spawn(async move {
        loop {
            let cur = handle_1.get("cur").unwrap();
            log::info!("loop 1 get {}", cur.0);
            drop(cur);
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    });

    let loop_2 = tokio::spawn(async move {
        let mut count = 0i32;
        loop {
            handle_2.insert("cur", MyValue(count));
            count += 1;
            // let cur_ref = handle_2.get("cur").unwrap();
            // let cur = cur_ref.value().clone();
            // drop(cur_ref);
            // println!("{:?}", cur);
            // tokio::time::sleep(Duration::from_millis(10)).await;
        }
    });

    tokio::select! {
        Err(err) = loop_1 => {
            log::error!("loop_1: {}", err)
        },
        Err(err) = loop_2 => {
            log::error!("loop_2: {}", err)
        },
    }
    Ok(())
}
