use crate::client::SftpClientHandle;
use std::ffi::{CStr, CString, c_longlong};
use std::os::raw::c_char;

#[unsafe(no_mangle)]
pub extern "C" fn sftp_client_read_chunk(
    handle: *mut SftpClientHandle,
    path: *const c_char,
    offset: u64,
    buffer_ptr: *mut u8,
    buffer_len: usize,
    bytes_read_out: *mut usize,
) -> i32 {
    if handle.is_null() || path.is_null() || buffer_ptr.is_null() || bytes_read_out.is_null() {
        return -1;
    }

    let client = unsafe { &*handle };
    let path_str = unsafe { CStr::from_ptr(path).to_str().unwrap_or("") };

    let result = client.rt.block_on(async {
        use tokio::io::{AsyncReadExt, AsyncSeekExt};

        let mut file = client.sftp.open(path_str).await.map_err(|e| {
            eprintln!("Failed to open file for reading ('{}'): {:?}", path_str, e);
            -2
        })?;

        file.seek(std::io::SeekFrom::Start(offset))
            .await
            .map_err(|_| -3)?;

        let buffer_slice = unsafe { std::slice::from_raw_parts_mut(buffer_ptr, buffer_len) };
        let n = file.read(buffer_slice).await.map_err(|e| {
            eprintln!("Failed to read data at offset {}: {:?}", offset, e);
            -4
        })?;

        Ok::<usize, i32>(n)
    });

    match result {
        Ok(n) => {
            unsafe {
                *bytes_read_out = n;
            }
            0
        }
        Err(code) => code,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn sftp_client_write_chunk(
    handle: *mut SftpClientHandle,
    path: *const c_char,
    offset: u64,
    data_ptr: *const u8,
    data_len: usize,
) -> i32 {
    if handle.is_null() || path.is_null() || data_ptr.is_null() {
        return -1;
    }

    let client = unsafe { &*handle };
    let path_str = unsafe { CStr::from_ptr(path).to_str().unwrap_or("") };

    let data_slice = unsafe { std::slice::from_raw_parts(data_ptr, data_len) };
    let data_vec = data_slice.to_vec();

    let result = client.rt.block_on(async {
        use russh_sftp::protocol::OpenFlags;
        use tokio::io::{AsyncSeekExt, AsyncWriteExt};

        let mut file = if offset == 0 {
            client.sftp.create(path_str).await.map_err(|e| {
                eprintln!(
                    "Failed to create file at offset 0 ('{}'): {:?}",
                    path_str, e
                );
                -2
            })?
        } else {
            client
                .sftp
                .open_with_flags(path_str, OpenFlags::WRITE | OpenFlags::CREATE)
                .await
                .map_err(|e| {
                    eprintln!(
                        "Failed to open file for writing at offset {} ('{}'): {:?}",
                        offset, path_str, e
                    );
                    -2
                })?
        };

        file.seek(std::io::SeekFrom::Start(offset))
            .await
            .map_err(|_| -3)?;

        file.write_all(&data_vec).await.map_err(|e| {
            eprintln!("Failed to write chunk data: {:?}", e);
            -4
        })?;

        Ok::<(), i32>(())
    });

    match result {
        Ok(_) => 0,
        Err(code) => code,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn sftp_client_mkdir(handle: *mut SftpClientHandle, path: *const c_char) -> i32 {
    if handle.is_null() || path.is_null() {
        return -1;
    }

    let client = unsafe { &*handle };
    let path_str = unsafe { CStr::from_ptr(path).to_str().unwrap_or("") };

    let result = client.rt.block_on(async {
        match client.sftp.create_dir(path_str).await {
            Ok(_) => Ok::<(), i32>(()),
            Err(_) => Ok(()), // Treat directory existence as relative success
        }
    });

    match result {
        Ok(_) => 0,
        Err(code) => code,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn sftp_client_rmdir(handle: *mut SftpClientHandle, path: *const c_char) -> i32 {
    if handle.is_null() || path.is_null() {
        return -1;
    }

    let client = unsafe { &*handle };
    let path_str = unsafe { CStr::from_ptr(path).to_str().unwrap_or("") };

    let result = client.rt.block_on(async {
        client.sftp.remove_dir(path_str).await.map_err(|_| -2)?;
        Ok::<(), i32>(())
    });

    match result {
        Ok(_) => 0,
        Err(code) => code,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn sftp_client_remove_file(
    handle: *mut SftpClientHandle,
    path: *const c_char,
) -> i32 {
    if handle.is_null() || path.is_null() {
        return -1;
    }

    let client = unsafe { &*handle };
    let path_str = unsafe { CStr::from_ptr(path).to_str().unwrap_or("") };

    let result = client.rt.block_on(async {
        client.sftp.remove_file(path_str).await.map_err(|_| -2)?;
        Ok::<(), i32>(())
    });

    match result {
        Ok(_) => 0,
        Err(code) => code,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn sftp_client_list_dir(
    handle: *mut SftpClientHandle,
    path: *const c_char,
) -> *mut c_char {
    if handle.is_null() || path.is_null() {
        return std::ptr::null_mut();
    }

    let client = unsafe { &*handle };
    let path_str = unsafe { CStr::from_ptr(path).to_str().unwrap_or("") };

    let result = client.rt.block_on(async {
        let mut entries = client.sftp.read_dir(path_str).await.map_err(|_| -2)?;
        let mut names = Vec::new();

        while let Some(item) = entries.next() {
            let name = item.file_name();
            names.push(name.to_string());
        }

        Ok::<String, i32>(names.join("\n"))
    });

    match result {
        Ok(list_str) => match CString::new(list_str) {
            Ok(c_str) => c_str.into_raw(),
            Err(_) => std::ptr::null_mut(),
        },
        Err(_) => std::ptr::null_mut(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn sftp_client_free_string(s: *mut c_char) {
    if !s.is_null() {
        unsafe {
            let _ = CString::from_raw(s);
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn sftp_client_file_size(
    handle: *mut SftpClientHandle,
    path: *const c_char,
    size_out: *mut c_longlong,
) -> i32 {
    if handle.is_null() || path.is_null() || size_out.is_null() {
        return -1;
    }

    let client = unsafe { &*handle };
    let path_str = unsafe { CStr::from_ptr(path).to_str().unwrap_or("") };

    let result = client.rt.block_on(async {
        let metadata = client.sftp.metadata(path_str).await.map_err(|e| {
            eprintln!("Failed to retrieve metadata for '{}': {:?}", path_str, e);
            -2
        })?;

        let len = metadata.len();
        Ok::<u64, i32>(len)
    });

    match result {
        Ok(sz) => {
            unsafe {
                *size_out = sz as c_longlong;
            }
            0
        }
        Err(code) => code,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn sftp_client_exists(handle: *mut SftpClientHandle, path: *const c_char) -> i32 {
    if handle.is_null() || path.is_null() {
        return -1;
    }

    let client = unsafe { &*handle };
    let path_str = unsafe { CStr::from_ptr(path).to_str().unwrap_or("") };

    let exists = client.rt.block_on(async {
        match client.sftp.metadata(path_str).await {
            Ok(_) => true,
            Err(_) => false,
        }
    });

    if exists { 1 } else { 0 }
}

#[unsafe(no_mangle)]
pub extern "C" fn sftp_client_rename(
    handle: *mut SftpClientHandle,
    old_path: *const c_char,
    new_path: *const c_char,
) -> i32 {
    if handle.is_null() || old_path.is_null() || new_path.is_null() {
        return -1;
    }

    let client = unsafe { &*handle };
    let old_str = unsafe { CStr::from_ptr(old_path).to_str().unwrap_or("") };
    let new_str = unsafe { CStr::from_ptr(new_path).to_str().unwrap_or("") };

    let result = client.rt.block_on(async {
        client.sftp.rename(old_str, new_str).await.map_err(|e| {
            eprintln!(
                "Failed to rename remote path from '{}' to '{}': {:?}",
                old_str, new_str, e
            );
            -2
        })?;
        Ok::<(), i32>(())
    });

    match result {
        Ok(_) => 0,
        Err(code) => code,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn sftp_client_stat(
    handle: *mut SftpClientHandle,
    path: *const c_char,
    size_out: *mut c_longlong,
    is_dir_out: *mut i32,
    permissions_out: *mut u32,
    uid_out: *mut u32,
    gid_out: *mut u32,
    mtime_out: *mut u64,
) -> i32 {
    if handle.is_null()
        || path.is_null()
        || size_out.is_null()
        || is_dir_out.is_null()
        || permissions_out.is_null()
        || uid_out.is_null()
        || gid_out.is_null()
        || mtime_out.is_null()
    {
        return -1;
    }

    let client = unsafe { &*handle };
    let path_str = unsafe { CStr::from_ptr(path).to_str().unwrap_or("") };

    let result = client.rt.block_on(async {
        let metadata = client.sftp.metadata(path_str).await.map_err(|e| {
            eprintln!("Failed to get metadata for '{}': {:?}", path_str, e);
            -2
        })?;

        let size = metadata.len() as c_longlong;
        let is_dir = if metadata.is_dir() { 1 } else { 0 };

        let permissions = metadata.permissions.unwrap_or(0);
        let uid = metadata.uid.unwrap_or(0);
        let gid = metadata.gid.unwrap_or(0);
        let mtime = metadata.mtime.unwrap_or(0) as u64;

        Ok::<(c_longlong, i32, u32, u32, u32, u64), i32>((
            size,
            is_dir,
            permissions,
            uid,
            gid,
            mtime,
        ))
    });

    match result {
        Ok((sz, dir_flag, perm, u, g, mt)) => {
            unsafe {
                *size_out = sz;
                *is_dir_out = dir_flag;
                *permissions_out = perm;
                *uid_out = u;
                *gid_out = g;
                *mtime_out = mt;
            }
            0
        }
        Err(code) => code,
    }
}
