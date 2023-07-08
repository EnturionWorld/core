use enturion_shared::signals::{Signal, Signals};
use enturion_shared::RUNTIME;
use std::time::Duration;
use tokio::runtime::Runtime;
use tokio::time;

type TickCallback = unsafe extern "C" fn();
extern "C" {
    pub fn AbortHandler();
    pub fn World_IsStopped() -> i32;
}

async fn async_main(tick_callback: TickCallback) -> Result<(), Box<dyn std::error::Error>> {
    let mut interval = time::interval(Duration::from_millis(5));
    let mut signals = Signals::default();

    loop {
        tokio::select! {
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

                if unsafe { World_IsStopped() } > 0 {
                    break;
                }
            }
        }
    }

    Ok(())
}

#[no_mangle]
pub extern "C" fn WorldServerRsInit() {
    // Create the runtime
    let _ = RUNTIME.set(Runtime::new().unwrap());
}

#[no_mangle]
pub extern "C" fn WorldServerRsMain(tick_callback: TickCallback) {
    let rt = RUNTIME.get().unwrap();
    let main_handle = rt.spawn(async move {
        async_main(tick_callback).await.unwrap();
    });

    let _ = rt.block_on(main_handle);
}
