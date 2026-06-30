mod vhd;
mod partition;

pub use vhd::{create_vhd, mount_and_get_handle, detach_with_handle, get_physical_path, format_vhd};
pub use partition::create_two_partitions;