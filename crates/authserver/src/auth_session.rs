use anyhow::Result;
use bincode::config as bincode_config;
use bincode::config::Configuration;
use bytes::{Buf, Bytes, BytesMut};
use enturion_shared::net::{Session, WoWPacket};
use enturion_shared::AsyncResult;
use log::{error, trace};
use std::ffi::{c_char, c_void, CString};
use std::mem::MaybeUninit;
use std::net::SocketAddr;
use std::pin::Pin;
use std::ptr::slice_from_raw_parts;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::TcpStream;
use tokio::runtime::Handle;
use tokio::time;

extern "C" {
    fn AuthSession_Free(auth_session: *mut c_void);
    fn AuthSession_New(rs_auth_session: *mut c_void) -> *mut c_void;
    fn AuthSession_Start(auth_session: *const c_void);
    fn AuthSession_Update(auth_session: *const c_void);
    fn AuthSession_WriteIntoBuffer(auth_session: *const c_void, data: *const c_void, size: usize);
}

struct CxxAuthSession(*const c_void);
unsafe impl Send for CxxAuthSession {}
impl CxxAuthSession {
    fn write_into_buffer(&self, data: Bytes) {
        unsafe {
            AuthSession_WriteIntoBuffer(self.0, std::mem::transmute(data.as_ptr()), data.len());
        }
    }
}

pub struct AuthSession {
    rx: OwnedReadHalf,
    tx: OwnedWriteHalf,
    socket_address: SocketAddr,
    socket_address_as_str: CString,
    cxx_auth_session: MaybeUninit<CxxAuthSession>,
    should_disconnect: AtomicBool,
    should_shutdown: AtomicBool,
    bincode_configuration: Configuration,
}

impl AuthSession {
    pub fn new(stream: TcpStream, address: SocketAddr) -> Pin<Box<Self>> {
        let (rx, tx) = stream.into_split();
        let address_as_string = match address {
            SocketAddr::V4(addr) => addr.ip().to_string(),
            SocketAddr::V6(addr) => addr.ip().to_string(),
        };

        let result = Self {
            rx,
            tx,
            socket_address: address,
            socket_address_as_str: CString::new(address_as_string).unwrap(),
            cxx_auth_session: MaybeUninit::uninit(),
            should_disconnect: AtomicBool::new(false),
            should_shutdown: AtomicBool::new(false),
            bincode_configuration: bincode_config::standard().with_little_endian(),
        };

        let mut boxed = Box::pin(result);

        let cxx_auth_session = unsafe { AuthSession_New(std::mem::transmute(boxed.as_ref())) };
        let _ = boxed
            .cxx_auth_session
            .write(CxxAuthSession(cxx_auth_session));

        boxed
    }

    pub async fn start(&mut self) -> Result<()> {
        trace!(target: "session", "Starting session for {}", self.socket_address);
        unsafe { AuthSession_Start(self.cxx_auth_session.assume_init_read().0) };

        let mut buf = BytesMut::with_capacity(4096);
        let mut interval = time::interval(Duration::from_millis(5));

        loop {
            tokio::select! {
                result = self.rx.read_buf(&mut buf) => {
                    let n = match result {
                        Ok(n) if n == 0 => return Ok(()),
                        Ok(n) => {
                            if self.should_shutdown.load(Ordering::Relaxed) {
                                let _ = self.tx.flush().await;
                                0
                            } else {
                                n
                            }
                        },
                        Err(e) => {
                            error!(target: "session", "Failed to read from socket. Err = {}", e);
                            return Err(e.into());
                        }
                    };

                    trace!(target: "session", "Received {} bytes", n);
                    unsafe { self.cxx_auth_session.assume_init_read() }.write_into_buffer(buf.copy_to_bytes(n));
                },
                _ = interval.tick() => {
                    unsafe { AuthSession_Update(self.cxx_auth_session.assume_init_read().0); }
                    if self.should_disconnect.load(Ordering::Relaxed) {
                        let _ = self.tx.flush().await;
                        let _ = self.tx.shutdown().await;
                    }
                }
            }
        }
    }

    pub fn get_ip_address(&self) -> &SocketAddr {
        &self.socket_address
    }

    pub fn disconnect(&self) {
        self.should_disconnect.store(true, Ordering::SeqCst);
    }

    pub fn shutdown(&self) {
        self.should_shutdown.store(true, Ordering::SeqCst);
    }

    #[no_mangle]
    pub unsafe extern "C" fn AuthSession_GetRemoteIpAddress(this: *const c_void) -> *const c_char {
        let this_obj = std::mem::transmute::<_, &Self>(this);
        this_obj.socket_address_as_str.as_ptr()
    }

    #[no_mangle]
    pub unsafe extern "C" fn AuthSession_GetRemotePort(this: *const c_void) -> u16 {
        let this_obj = std::mem::transmute::<_, &Self>(this);
        this_obj.socket_address.port()
    }

    #[no_mangle]
    pub unsafe extern "C" fn AuthSession_WritePacket(
        this: *const c_void,
        data: *const u8,
        size: usize,
    ) {
        let this_obj = std::mem::transmute::<_, &mut Self>(this);
        let buf = slice_from_raw_parts(data, size);
        let buf = buf.as_ref().unwrap().to_vec();

        Handle::try_current().unwrap().spawn(async move {
            let res = this_obj.tx.write_all(buf.as_slice()).await;

            if let Err(e) = res {
                error!(target: "session", "Error writing packet to tcp stream: {}", e.to_string());
                this_obj.disconnect();
            }
        });
    }

    #[no_mangle]
    pub unsafe extern "C" fn AuthSession_Disconnect(this: *const c_void) {
        let this_obj = std::mem::transmute::<_, &Self>(this);
        this_obj.disconnect();
    }

    #[no_mangle]
    pub unsafe extern "C" fn AuthSession_Shutdown(this: *const c_void) {
        let this_obj = std::mem::transmute::<_, &Self>(this);
        this_obj.shutdown();
    }
}

impl Session for AuthSession {
    fn send_packet<'a, T: WoWPacket + Send + 'a>(&'a mut self, pkt: T) -> AsyncResult<()> {
        Box::pin(async move {
            let v = bincode::encode_to_vec(pkt, self.bincode_configuration)?;
            Ok(self.tx.write_all(v.as_slice()).await?)
        })
    }
}

impl Drop for AuthSession {
    fn drop(&mut self) {
        trace!(target: "session", "Connection to {} closed.", self.get_ip_address());
        unsafe { AuthSession_Free(self.cxx_auth_session.assume_init_read().0 as *mut c_void) };
    }
}
