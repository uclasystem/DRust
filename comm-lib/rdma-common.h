#ifndef RDMA_COMMON_H
#define RDMA_COMMON_H

#include <netdb.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <rdma/rdma_cma.h>

#include <sys/mman.h>
#include <arpa/inet.h>

#define TEST_NZ(x)                                      \
  do                                                    \
  {                                                     \
    if ((x))                                            \
      die("error: " #x " failed (returned non-zero)."); \
  } while (0)
#define TEST_Z(x)                                        \
  do                                                     \
  {                                                      \
    if (!(x))                                            \
      die("error: " #x " failed (returned zero/null)."); \
  } while (0)
#define TOTAL_NUM_SERVERS 8
#define NUM_SERVERS (TOTAL_NUM_SERVERS - 1)
#define THREAD_FLAG_NUM (1024 * 1024)

extern struct rdma_cm_id *global_conn[NUM_SERVERS + 2];
extern size_t remote_region_start_addr;
extern size_t region_size;
extern size_t cur_id;
extern char *mem_region;

enum mode
{
  M_WRITE,
  M_READ
};

void die(const char *reason);

void build_connection(struct rdma_cm_id *id);
void build_params(struct rdma_conn_param *params);
void destroy_connection(void *context);
// void * get_local_message_region(void *context);
void on_connect(void *context);
void send_mr(void *context);
void set_mode(enum mode m);
char *drust_write_impl(size_t local_src_offset, size_t remote_dst_offset, size_t byte_size, struct rdma_cm_id *conn_id);
char *drust_read_impl(size_t local_dst_offset, size_t remote_src_offset, size_t byte_size, struct rdma_cm_id *conn_id, bool reading_flag);
char *drust_xchg_impl(size_t local_src_offset, size_t remote_dst_offset, uint64_t compare_value, uint64_t swap_value, struct rdma_cm_id *conn_id);
char *drust_fetch_add_impl(size_t local_src_offset, size_t remote_dst_offset, uint64_t add_value, struct rdma_cm_id *conn_id);

char *drust_xchg_local_impl(size_t local_src_offset, size_t remote_dst_offset, uint64_t compare_value, uint64_t swap_value, struct rdma_cm_id *conn_id);
char *drust_local_read_impl(size_t local_dst_offset, size_t remote_src_offset, size_t byte_size, struct rdma_cm_id *conn_id, bool reading_flag);

void wait_until_connection_established(struct rdma_cm_id *conn_id);

#endif
