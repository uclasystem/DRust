pub const MATRIX_SIZE: usize = 32768;
pub const SINGLE_SIZE: usize = 16;

pub const THREADS_NUM: usize = 16;
pub const BRANCH_NUM: usize = 21;


extern "C" {
    pub fn drust_start_server(heap_start: usize, heap_size: usize, server_id: usize) -> i32;
    pub fn drust_server_ready();
    pub fn drust_disconnect() -> i32;
    pub fn copy_mem(src_addr: usize, dst_addr: usize, byte_size: usize) -> i32;
    pub fn register_mem(heap_start: usize, heap_size: usize) -> usize;
    pub fn drust_write(
        local_src_offset: usize,
        remote_dst_offset: usize,
        byte_size: usize,
    ) -> usize;
    pub fn drust_read(local_dst_offset: usize, remote_src_offset: usize, byte_size: usize)
        -> usize;
    pub fn drust_write_sync(
        local_src_offset: usize,
        remote_dst_offset: usize,
        byte_size: usize,
        thread_flag_id: usize,
    ) -> usize;
    pub fn drust_read_sync(
        local_dst_offset: usize,
        remote_src_offset: usize,
        byte_size: usize,
        thread_flag_id: usize,
    ) -> usize;
    pub fn drust_atomic_cmp_exchg(
        local_src_offset: usize,
        remote_dst_offset: usize,
        old_value: usize,
        new_value: usize,
    ) -> usize;
    pub fn drust_atomic_cmp_exchg_sync(
        local_src_offset: usize,
        remote_dst_offset: usize,
        old_value: usize,
        new_value: usize,
        thread_flag_id: usize,
    ) -> usize;
    pub fn drust_atomic_fetch_add(
        local_src_offset: usize,
        remote_dst_offset: usize,
        add_value: usize,
    ) -> usize;
    pub fn drust_atomic_fetch_add_sync(
        local_src_offset: usize,
        remote_dst_offset: usize,
        add_value: usize,
        thread_flag_id: usize,
    ) -> usize;
    // pub fn drust_local_atomic_cmp_exchg_sync(
    //   local_src_offset: usize,
    //   remote_dst_offset: usize,
    //   old_value: usize,
    //   new_value: usize,
    //   thread_flag_id: usize
    // ) -> usize;
}
