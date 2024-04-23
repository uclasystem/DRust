use std::{net::SocketAddr, time::Duration};

use tokio::{runtime::Runtime, time::sleep};

use crate::{conf::NUM_SERVERS, drust_std::connect::dsafepoint::{rshutdown, rsync, set_ready}};

// load column from file and return a Column struct
pub async fn drust_main(
    app_addrs: [SocketAddr; NUM_SERVERS],
    safepoint_addrs: [SocketAddr; NUM_SERVERS],
    server_idx: usize,
) {
    // std::thread::spawn(move || {
    //     Runtime::new().unwrap().block_on(run_server(server_addr[server_idx]));
    // });
    sleep(Duration::from_secs(2)).await;
    set_ready(2);

    // let mut guess = String::new();
    // println!("RDMA connected, press enter to initialize the application clients");
    // io::stdin().read_line(&mut guess).expect("failed to readline");
    rsync(&safepoint_addrs, 1).await;
    rsync(&safepoint_addrs, 2).await;
    // dconnect!(server_addr, DCLIENTS, ColumnWorldClient);
    if server_idx == 0 {
        println!("drust_main done");
        rshutdown(&safepoint_addrs).await;
    }
}
