use sftpray::*;
use std::ffi::CString;

#[test]
fn test_sftp_secondary_utilities() {
    let host = CString::new("127.0.0.1").unwrap();
    let user = CString::new("foo").unwrap();
    let pass = CString::new("pass").unwrap();

    let handle = sftp_client_new(host.as_ptr(), 2222, user.as_ptr(), pass.as_ptr());
    assert!(!handle.is_null(), "The client handle must not be null");

    let original_path = CString::new("upload/utils_test.txt").unwrap();
    let renamed_path = CString::new("upload/utils_renamed.txt").unwrap();
    let data = b"FFI secondary utilities test";

    // Write initial file
    let w = sftp_client_write_chunk(handle, original_path.as_ptr(), 0, data.as_ptr(), data.len());
    assert_eq!(w, 0, "Failed to write initial file chunk");

    // Test Existence
    assert_eq!(
        sftp_client_exists(handle, original_path.as_ptr()),
        1,
        "File should exist"
    );

    // Test File Size
    let mut size: i64 = 0;
    assert_eq!(
        sftp_client_file_size(handle, original_path.as_ptr(), &mut size),
        0,
        "Failed to retrieve file size"
    );
    assert_eq!(
        size as usize,
        data.len(),
        "File size does not match data length"
    );

    // Test Rename
    assert_eq!(
        sftp_client_rename(handle, original_path.as_ptr(), renamed_path.as_ptr()),
        0,
        "Failed to rename file"
    );
    assert_eq!(
        sftp_client_exists(handle, original_path.as_ptr()),
        0,
        "Original file should no longer exist"
    );
    assert_eq!(
        sftp_client_exists(handle, renamed_path.as_ptr()),
        1,
        "Renamed file should exist"
    );

    // Cleanup
    let _ = sftp_client_remove_file(handle, renamed_path.as_ptr());
    sftp_client_free(handle);
}
