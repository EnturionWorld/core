extern crate self as enturion_authserver;

mod auth_session;
pub(crate) mod packet;

use crate::auth_session::AuthSession;
use anyhow::Result;
use enturion_shared::config::Config;
use enturion_shared::signals::{Signal, Signals};
use enturion_shared::RUNTIME;
use log::trace;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::runtime::Runtime;
use tokio::time;

type TickCallback = unsafe extern "C" fn();
extern "C" {
    pub fn AbortHandler();
    pub fn ConfigGetInstance() -> &'static Config;
}

fn get_bind_addr() -> Result<(String, u16)> {
    let config = unsafe { ConfigGetInstance() };
    let addr = config.get("BindIp", Some("0.0.0.0"))?;
    let port = config.get("RealmServerPort", Some(3724_u16))?;

    Ok((addr.to_string(), port))
}

async fn async_main(tick_callback: TickCallback) -> Result<()> {
    let listener = TcpListener::bind(get_bind_addr()?).await?;

    let mut interval = time::interval(Duration::from_millis(5));
    let mut signals = Signals::default();

    loop {
        tokio::select! {
            Ok((tcp_stream, socket_addr)) = listener.accept() => {
                trace!(target: "session", "Accepting incoming connection from {}", socket_addr);
                let mut session = AuthSession::new(tcp_stream, socket_addr);
                let _ = tokio::spawn(async move {
                    let _ = session.start().await;
                });
            },
            signal = signals.as_mut() => {
                match signal {
                    Some(Signal::Abort) => {
                        unsafe { AbortHandler(); }
                    },
                    _ => {
                        break;
                    },
                }
            },
            _ = interval.tick() => {
                unsafe { tick_callback(); }
                ::log::logger().flush();
            }
        }
    }

    Ok(())
}

#[no_mangle]
pub extern "C" fn AuthServerRsInit() {
    // Create the runtime
    let _ = RUNTIME.set(Runtime::new().unwrap());
}

#[no_mangle]
pub extern "C" fn AuthServerRsMain(tick_callback: TickCallback) {
    let rt = RUNTIME.get().unwrap();
    let main_handle = rt.spawn(async move {
        async_main(tick_callback).await.unwrap();
    });

    let _ = rt.block_on(main_handle);
}
