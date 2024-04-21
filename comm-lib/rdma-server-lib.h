#ifndef RDMA_SERVER_H
#define RDMA_SERVER_H
#include "rdma-common.h"

int drust_start_server(size_t heap_start, size_t heap_size, size_t server_id);
void drust_server_ready(void);

size_t drust_write(size_t local_src_offset, size_t dst_offset, size_t byte_size);
size_t drust_write_sync(size_t local_src_offset, size_t dst_offset, size_t byte_size, size_t thread_flag_id);
size_t drust_read(size_t local_dst_offset, size_t src_offset, size_t byte_size);
size_t drust_read_sync(size_t local_dst_offset, size_t src_offset, size_t byte_size, size_t thread_flag_id);

size_t drust_atomic_cmp_exchg(size_t local_src_offset, size_t remote_dst_offset, size_t old_value, size_t new_value);
size_t drust_atomic_cmp_exchg_sync(size_t local_src_offset, size_t remote_dst_offset, size_t old_value, size_t new_value, size_t thread_flag_id);
size_t drust_atomic_fetch_add(size_t local_src_offset, size_t remote_dst_offset, size_t add_value);
size_t drust_atomic_fetch_add_sync(size_t local_src_offset, size_t remote_dst_offset, size_t add_value, size_t thread_flag_id);
size_t drust_local_atomic_cmp_exchg_sync(size_t local_src_offset, size_t remote_dst_offset, size_t old_value, size_t new_value, size_t thread_flag_id);

int copy_mem(size_t src_addr, size_t dst_addr, size_t byte_size);
size_t register_mem(size_t heap_start, size_t heap_size);
#endif