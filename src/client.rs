use russh::client;
use russh::keys::PublicKeyOrCertificate;
use russh_sftp::client::SftpSession;
use std::ffi::{CStr, c_char};
use std::sync::Arc;
use tokio::runtime::Runtime;

struct ClientHandler {}

impl client::Handler for ClientHandler {
    type Error = russh::Error;

    fn check_server_key(
        &mut self,
        _server_public_key: &PublicKeyOrCertificate,
    ) -> impl std::future::Future<Output = Result<bool, Self::Error>> + Send {
        async { Ok(true) }
    }
}

pub struct SftpClientHandle {
    pub rt: Runtime,
    pub sftp: SftpSession,
}

#[unsafe(no_mangle)]
pub extern "C" fn sftp_client_new(
    host: *const c_char,
    port: u16,
    user: *const c_char,
    pass: *const c_char,
) -> *mut SftpClientHandle {
    if host.is_null() || user.is_null() || pass.is_null() {
        return std::ptr::null_mut();
    }

    let host_str = unsafe { CStr::from_ptr(host).to_str().ok() };
    let user_str = unsafe { CStr::from_ptr(user).to_str().ok() };
    let pass_str = unsafe { CStr::from_ptr(pass).to_str().ok() };

    let (host_str, user_str, pass_str) = match (host_str, user_str, pass_str) {
        (Some(h), Some(u), Some(p)) => (h, u, p),
        _ => return std::ptr::null_mut(),
    };

    let addr = format!("{}:{}", host_str, port);

    let rt = match Runtime::new() {
        Ok(rt) => rt,
        Err(_) => return std::ptr::null_mut(),
    };

    let sftp_result = rt.block_on(async move {
        let config = Arc::new(client::Config::default());
        let sh = ClientHandler {};

        let mut session = client::connect(config, &addr, sh).await.map_err(|_| -1)?;
        let auth = session
            .authenticate_password(user_str, pass_str)
            .await
            .map_err(|_| -2)?;
        if !auth.success() {
            return Err(-3);
        }

        let channel = session.channel_open_session().await.map_err(|_| -4)?;
        channel
            .request_subsystem(true, "sftp")
            .await
            .map_err(|_| -5)?;

        let sftp = SftpSession::new(channel.into_stream())
            .await
            .map_err(|_| -6)?;
        Ok(sftp)
    });

    let sftp = match sftp_result {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };

    let handle = SftpClientHandle { rt, sftp };
    Box::into_raw(Box::new(handle))
}

#[unsafe(no_mangle)]
pub extern "C" fn sftp_client_free(handle: *mut SftpClientHandle) {
    if !handle.is_null() {
        unsafe {
            let _ = Box::from_raw(handle);
        }
    }
}
