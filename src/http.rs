use crate::{http_server::start_http_server, structs::WmInnerSignals};
use alloc::rc::Rc;
use embassy_executor::Spawner;
use embassy_net::Stack;

// #[embassy_executor::task(pool_size = WEB_TASK_POOL_SIZE)]
// async fn web_task(stack: embassy_net::Stack<'static>, signals: Rc<WmInnerSignals>) {
//     let gw_ip_addr_str = "192.168.4.1";
//     start_http_server(stack, signals, gw_ip_addr_str).await;
// }

pub async fn run_http_server(
    spawner: &Spawner,
    ap_stack: Stack<'static>,
    signals: Rc<WmInnerSignals>,
) {
    // loop {
    //     if ap_stack.is_link_up() {
    //         log::info!("AP link up");
    //         break;
    //     }
    //     Timer::after(Duration::from_millis(500)).await;
    //     log::info!("AP link not up");
    // }
    // // let port = 80;

    // while !ap_stack.is_config_up() {
    //     Timer::after(Duration::from_millis(100)).await
    // }
    ap_stack
        .config_v4()
        .inspect(|c| log::info!("ipv4 config: {c:?}"));
    spawner.must_spawn(start_http_server(ap_stack, signals));
}
