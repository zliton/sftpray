use sftpray::*;
use std::ffi::CString;

#[test]
fn test_sftp_full_lifecycle() {
    let host = CString::new("127.0.0.1").unwrap();
    let user = CString::new("foo").unwrap();
    let pass = CString::new("pass").unwrap();

    let handle = sftp_client_new(host.as_ptr(), 2222, user.as_ptr(), pass.as_ptr());
    assert!(!handle.is_null(), "The client handle must not be null");

    let remote_file_path = CString::new("upload/sample.txt").unwrap();
    let chunk1 = b"test ";
    let chunk2 = b"chunks ";
    let chunk3 = b"with sftpray.\n";

    let w1 = sftp_client_write_chunk(
        handle,
        remote_file_path.as_ptr(),
        0,
        chunk1.as_ptr(),
        chunk1.len(),
    );
    assert_eq!(w1, 0, "Failed to write chunk 1");

    let w2 = sftp_client_write_chunk(
        handle,
        remote_file_path.as_ptr(),
        chunk1.len() as u64,
        chunk2.as_ptr(),
        chunk2.len(),
    );
    assert_eq!(w2, 0, "Failed to write chunk 2");

    let w3 = sftp_client_write_chunk(
        handle,
        remote_file_path.as_ptr(),
        (chunk1.len() + chunk2.len()) as u64,
        chunk3.as_ptr(),
        chunk3.len(),
    );
    assert_eq!(w3, 0, "Failed to write chunk 3");

    let mut buffer = vec![0u8; 128];
    let mut bytes_read: usize = 0;
    let read_status = sftp_client_read_chunk(
        handle,
        remote_file_path.as_ptr(),
        0,
        buffer.as_mut_ptr(),
        buffer.len(),
        &mut bytes_read,
    );
    assert_eq!(read_status, 0, "Failed to read file chunk");

    let final_content = String::from_utf8_lossy(&buffer[..bytes_read]).to_string();
    assert_eq!(final_content, "test chunks with sftpray.\n");

    let _ = sftp_client_remove_file(handle, remote_file_path.as_ptr());
    sftp_client_free(handle);
}
