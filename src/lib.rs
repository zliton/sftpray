pub mod client;
pub mod ops;

pub use client::SftpClientHandle;
pub use client::{sftp_client_free, sftp_client_new};
pub use ops::{
    sftp_client_exists, sftp_client_file_size, sftp_client_free_string, sftp_client_list_dir,
    sftp_client_mkdir, sftp_client_read_chunk, sftp_client_remove_file, sftp_client_rename,
    sftp_client_rmdir, sftp_client_write_chunk,
};
