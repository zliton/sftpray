use sftpray::*;
use std::ffi::CString;
use std::time::Instant;

fn run_sftp_benchmark(
    test_name: &str,
    total_size: usize,
    chunk_buf_size: usize,
    remote_filename: &str,
) {
    let host = CString::new("127.0.0.1").unwrap();
    let user = CString::new("foo").unwrap();
    let pass = CString::new("pass").unwrap();

    let handle = sftp_client_new(host.as_ptr(), 2222, user.as_ptr(), pass.as_ptr());
    assert!(!handle.is_null(), "[{}] Handle must not be null", test_name);

    let remote_path = CString::new(format!("upload/{}", remote_filename)).unwrap();

    // Generate deterministic test data
    let mut payload = vec![0u8; total_size];
    for (i, byte) in payload.iter_mut().enumerate() {
        *byte = ((i * 31) % 256) as u8;
    }

    println!("\n==============================================");
    println!(" [BENCHMARK] {}", test_name);
    println!(
        " File: {} KB | Chunk: {} KB | Total Chunks: {}",
        total_size / 1024,
        chunk_buf_size / 1024,
        (total_size + chunk_buf_size - 1) / chunk_buf_size
    );
    println!("==============================================");

    // --- 1. UPLOAD ---
    let upload_start = Instant::now();
    let mut offset: u64 = 0;

    for chunk in payload.chunks(chunk_buf_size) {
        let w_res = sftp_client_write_chunk(
            handle,
            remote_path.as_ptr(),
            offset,
            chunk.as_ptr(),
            chunk.len(),
        );
        assert_eq!(
            w_res, 0,
            "[{}] Failed to write chunk at offset {}",
            test_name, offset
        );
        offset += chunk.len() as u64;
    }
    let upload_duration = upload_start.elapsed();
    let upload_speed = (total_size as f64 / (1024.0 * 1024.0)) / upload_duration.as_secs_f64();

    println!(
        " -> UPLOAD   -> Time: {:?} | Velocity: {:.2} MB/s",
        upload_duration, upload_speed
    );

    // --- 2. DOWNLOAD ---
    let download_start = Instant::now();
    let mut downloaded_data = vec![0u8; total_size];
    let mut total_read = 0;
    let mut read_offset: u64 = 0;

    while total_read < total_size {
        let mut bytes_read: usize = 0;
        let read_res = sftp_client_read_chunk(
            handle,
            remote_path.as_ptr(),
            read_offset,
            downloaded_data[total_read..].as_mut_ptr(),
            chunk_buf_size,
            &mut bytes_read,
        );

        assert_eq!(
            read_res, 0,
            "[{}] Failed to read chunk at offset {}",
            test_name, read_offset
        );
        if bytes_read == 0 {
            break;
        }
        total_read += bytes_read;
        read_offset += bytes_read as u64;
    }

    let download_duration = download_start.elapsed();
    let download_speed = (total_size as f64 / (1024.0 * 1024.0)) / download_duration.as_secs_f64();

    println!(
        " -> DOWNLOAD -> Time: {:?} | Velocity: {:.2} MB/s",
        download_duration, download_speed
    );

    // Verify integrity
    assert_eq!(
        payload, downloaded_data,
        "[{}] Downloaded data does not match payload",
        test_name
    );

    // Cleanup
    let _ = sftp_client_remove_file(handle, remote_path.as_ptr());
    sftp_client_free(handle);
}

#[test]
fn test_bench_small_chunks_4kb() {
    run_sftp_benchmark(
        "Small Chunks (4 KB)",
        500 * 1024,
        4 * 1024,
        "bench_500kb.bin",
    );
}

#[test]
fn test_bench_optimal_chunks_32kb() {
    run_sftp_benchmark(
        "Optimal Chunks (32 KB)",
        2 * 1024 * 1024,
        32 * 1024,
        "bench_2mb.bin",
    );
}

#[test]
fn test_bench_large_chunks_128kb() {
    run_sftp_benchmark(
        "Large Chunks (128 KB)",
        5 * 1024 * 1024,
        128 * 1024,
        "bench_5mb.bin",
    );
}

#[test]
fn test_bench_extreme_chunks_256kb() {
    run_sftp_benchmark(
        "Extreme Chunks (256 KB)",
        10 * 1024 * 1024,
        256 * 1024,
        "bench_10mb.bin",
    );
}

#[test]
fn test_bench_insane_chunks_512kb() {
    run_sftp_benchmark(
        "Massive Chunks (512 KB)",
        20 * 1024 * 1024,
        512 * 1024,
        "bench_20mb.bin",
    );
}

#[test]
fn test_bench_mega_chunk_1024kb() {
    run_sftp_benchmark(
        "1 MB Chunks",
        30 * 1024 * 1024,
        1024 * 1024,
        "bench_30mb.bin",
    );
}
