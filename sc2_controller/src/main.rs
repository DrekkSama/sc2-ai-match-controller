mod game;
mod logging;
mod player_seats;
mod routes;
mod websocket;
mod ws_routes;

use crate::logging::init_logs;
use crate::routes::open_player_seat;
use common::portpicker::pick_unused_port_in_range;
use tracing::info;

#[tokio::main]
async fn main() {
    let _guards = init_logs();

    let port1 = pick_unused_port_in_range(9000..10000)
        .expect("Could not allocate port for SC2 process 1");
    let port2 = pick_unused_port_in_range(9000..10000)
        .expect("Could not allocate port for SC2 process 2");

    let seat1 = open_player_seat(1, false, port1).await;
    let seat2 = open_player_seat(2, false, port2).await;

    let observer_enabled = std::env::var("OBSERVER_ENABLED")
        .map(|v| v == "true")
        .unwrap_or(false);

    let observer_handle = if observer_enabled {
        info!("Observer enabled, opening observer seat on Player 1's SC2 process");
        Some(open_player_seat(3, true, port1).await)
    } else {
        None
    };

    match (seat1, seat2) {
        (Ok(ws1), Ok(ws2)) => {
            info!("Player seats opened successfully.");

            match observer_handle {
                Some(Ok(ws3)) => {
                    tokio::select! {
                        _ = ws1 => info!("Player seat 1 exited."),
                        _ = ws2 => info!("Player seat 2 exited."),
                        _ = ws3 => info!("Observer seat exited."),
                    }
                }
                Some(Err(e)) => {
                    info!("Observer seat failed to open: {:?}", e);
                    tokio::select! {
                        _ = ws1 => info!("Player seat 1 exited."),
                        _ = ws2 => info!("Player seat 2 exited."),
                    }
                }
                None => {
                    tokio::select! {
                        _ = ws1 => info!("Player seat 1 exited."),
                        _ = ws2 => info!("Player seat 2 exited."),
                    }
                }
            }
        }
        (Err(e), _) | (_, Err(e)) => {
            panic!("Failed to start SC2: {:?}", e);
        }
    }
}
