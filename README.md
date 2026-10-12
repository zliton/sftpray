# sftpray

> **High-performance, memory-safe SFTP client library** built with Rust (`russh` & `russh_sftp`), featuring an $O(1)$ constant memory footprint and cross-language support via FFI.

---

## Features

* **Lightning Fast:** Reaches up to **102+ MB/s** upload speeds using optimized chunking.
* **Memory Efficient:** Strict $O(1)$ memory consumption with zero garbage collection overhead.
* **Multi-Language SDK:** Designed to be easily extended into Python, Node.js, PHP, C++, and Go through a clean C Foreign Function Interface (FFI).

---

## Usage

### 1. Create a new SFTP client

```rust
use std::ffi::CString;

let host = CString::new("127.0.0.1").unwrap();
let user = CString::new("foo").unwrap();
let pass = CString::new("pass").unwrap();

let handle = sftp_client_new(host.as_ptr(), 2222, user.as_ptr(), pass.as_ptr());
assert!(!handle.is_null(), "The client handle must not be null");

// Don't forget to free the handle later
sftp_client_free(handle);

```

### 2. Write chunks to a remote file

```rust
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

```

### 3. Read chunks from the SFTP server

```rust
let mut buffer = vec![0u8; 128]; // 128 bytes buffer  
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

```

---

## Benchmark Performance

Tested on a local server environment using the command:

```bash
cargo test --test benchmark_test -- --nocapture --test-threads=1

```

```text
running 6 tests
test test_bench_extreme_chunks_256kb ...
==============================================
 [BENCHMARK] Extreme Chunks (256 KB)
 File: 10240 KB | Chunk: 256 KB | Total Chunks: 40
==============================================
 -> UPLOAD   -> Time: 116.03073ms | Velocity: 86.18 MB/s
 -> DOWNLOAD -> Time: 561.389232ms | Velocity: 17.81 MB/s
ok
test test_bench_insane_chunks_512kb ...
==============================================
 [BENCHMARK] Massive Chunks (512 KB)
 File: 20480 KB | Chunk: 512 KB | Total Chunks: 40
==============================================
 -> UPLOAD   -> Time: 208.605537ms | Velocity: 95.87 MB/s
 -> DOWNLOAD -> Time: 1.132588631s | Velocity: 17.66 MB/s
ok
test test_bench_large_chunks_128kb ...
==============================================
 [BENCHMARK] Large Chunks (128 KB)
 File: 5120 KB | Chunk: 128 KB | Total Chunks: 40
==============================================
 -> UPLOAD   -> Time: 79.798017ms | Velocity: 62.66 MB/s
 -> DOWNLOAD -> Time: 531.108728ms | Velocity: 9.41 MB/s
ok
test test_bench_mega_chunk_1024kb ...
==============================================
 [BENCHMARK] 1 MB Chunks
 File: 30720 KB | Chunk: 1024 KB | Total Chunks: 30
==============================================
 -> UPLOAD   -> Time: 293.198699ms | Velocity: 102.32 MB/s
 -> DOWNLOAD -> Time: 1.671075959s | Velocity: 17.95 MB/s
ok
test test_bench_optimal_chunks_32kb ...
==============================================
 [BENCHMARK] Optimal Chunks (32 KB)
 File: 2048 KB | Chunk: 32 KB | Total Chunks: 64
==============================================
 -> UPLOAD   -> Time: 58.213552ms | Velocity: 34.36 MB/s
 -> DOWNLOAD -> Time: 795.238542ms | Velocity: 2.51 MB/s
ok
test test_bench_small_chunks_4kb ...
==============================================
 [BENCHMARK] Small Chunks (4 KB)
 File: 500 KB | Chunk: 4 KB | Total Chunks: 125
==============================================
 -> UPLOAD   -> Time: 64.426801ms | Velocity: 7.58 MB/s
 -> DOWNLOAD -> Time: 1.334948454s | Velocity: 0.37 MB/s
ok

```
