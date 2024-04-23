#include "rdma-common.h"

// static const long long RDMA_BUFFER_SIZE = 1024*1024*1024*4ULL;
struct rdma_cm_id *global_conn[NUM_SERVERS + 2] = {NULL};
size_t remote_region_start_addr = 0;
size_t region_size = 0;
// the index for the total number of connection (active+passive)
size_t cur_id = 0;
char *mem_region = NULL;

struct message
{
  enum
  {
    MSG_MR,
    MSG_DONE
  } type;

  union
  {
    struct ibv_mr mr;
  } data;
};

struct context
{
  struct ibv_context *ctx;
  struct ibv_pd *pd;
  struct ibv_cq *cq;
  struct ibv_comp_channel *comp_channel;

  pthread_t cq_poller_thread;
};

struct connection
{
  struct rdma_cm_id *id;
  struct ibv_qp *qp;

  int connected;

  struct ibv_mr *recv_mr;
  struct ibv_mr *send_mr;
  // struct ibv_mr *rdma_local_mr;
  struct ibv_mr *rdma_remote_mr;

  struct ibv_mr peer_mr;

  struct message *recv_msg;
  struct message *send_msg;

  // char *rdma_local_region;
  char *rdma_remote_region;

  enum
  {
    SS_INIT,
    SS_MR_SENT,
    // SS_RDMA_SENT,
    SS_DONE_SENT
  } send_state;

  enum
  {
    RS_INIT,
    RS_MR_RECV,
    RS_DONE_RECV
  } recv_state;
};

static void build_context(struct ibv_context *verbs);
static void build_qp_attr(struct ibv_qp_init_attr *qp_attr);
// static char * get_peer_message_region(struct connection *conn);
static void on_completion(struct ibv_wc *);
static void *poll_cq(void *);
static void post_receives(struct connection *conn);
static void register_memory(struct connection *conn);
static void send_message(struct connection *conn);

static struct context *s_ctx[NUM_SERVERS + 2] = {NULL};
static enum mode s_mode = M_WRITE;

void die(const char *reason)
{
  fprintf(stderr, "%s\n", reason);
  exit(EXIT_FAILURE);
}

void build_connection(struct rdma_cm_id *id)
{
  struct connection *conn;
  struct ibv_qp_init_attr qp_attr;

  build_context(id->verbs);
  build_qp_attr(&qp_attr);

  TEST_NZ(rdma_create_qp(id, s_ctx[cur_id]->pd, &qp_attr));

  id->context = conn = (struct connection *)malloc(sizeof(struct connection));

  conn->id = id;
  conn->qp = id->qp;

  conn->send_state = SS_INIT;
  conn->recv_state = RS_INIT;

  conn->connected = 0;

  register_memory(conn);
  post_receives(conn);

  global_conn[cur_id] = id;

  struct ibv_device_attr attr;
  int rc;
  rc = ibv_query_device(id->verbs, &attr);
  if (rc == 0)
  {
    printf("atomic_cap: %d\n", attr.atomic_cap);
  }
}

void build_context(struct ibv_context *verbs)
{
  if (s_ctx[cur_id])
  {
    if (s_ctx[cur_id]->ctx != verbs)
      die("cannot handle events in more than one context.");

    return;
  }

  s_ctx[cur_id] = (struct context *)malloc(sizeof(struct context));

  s_ctx[cur_id]->ctx = verbs;

  TEST_Z(s_ctx[cur_id]->pd = ibv_alloc_pd(s_ctx[cur_id]->ctx));
  TEST_Z(s_ctx[cur_id]->comp_channel = ibv_create_comp_channel(s_ctx[cur_id]->ctx));
  TEST_Z(s_ctx[cur_id]->cq = ibv_create_cq(s_ctx[cur_id]->ctx, 8192, NULL, s_ctx[cur_id]->comp_channel, 0)); /* cqe=10 is arbitrary */
  TEST_NZ(ibv_req_notify_cq(s_ctx[cur_id]->cq, 0));

  TEST_NZ(pthread_create(&s_ctx[cur_id]->cq_poller_thread, NULL, poll_cq, NULL));
}

void build_params(struct rdma_conn_param *params)
{
  memset(params, 0, sizeof(*params));

  params->initiator_depth = params->responder_resources = 1;
  params->rnr_retry_count = 7; /* infinite retry */
}

void build_qp_attr(struct ibv_qp_init_attr *qp_attr)
{
  memset(qp_attr, 0, sizeof(*qp_attr));

  qp_attr->send_cq = s_ctx[cur_id]->cq;
  qp_attr->recv_cq = s_ctx[cur_id]->cq;
  qp_attr->qp_type = IBV_QPT_RC;

  qp_attr->cap.max_send_wr = 8192;
  qp_attr->cap.max_recv_wr = 8192;
  qp_attr->cap.max_send_sge = 1;
  qp_attr->cap.max_recv_sge = 1;
}

void destroy_connection(void *context)
{
  struct connection *conn = (struct connection *)context;

  rdma_destroy_qp(conn->id);

  ibv_dereg_mr(conn->send_mr);
  ibv_dereg_mr(conn->recv_mr);
  // ibv_dereg_mr(conn->rdma_local_mr);
  ibv_dereg_mr(conn->rdma_remote_mr);

  free(conn->send_msg);
  free(conn->recv_msg);
  // free(conn->rdma_local_region);
  // free(conn->rdma_remote_region);
  size_t local_flag_section_size = THREAD_FLAG_NUM * sizeof(char);
  size_t remote_flag_section_size = 4096 * sizeof(char);
  munmap(conn->rdma_remote_region, local_flag_section_size + region_size + remote_flag_section_size);

  rdma_destroy_id(conn->id);

  free(conn);
}

// void * get_local_message_region(void *context)
// {
//   if (s_mode == M_WRITE)
//     return ((struct connection *)context)->rdma_local_region;
//   else
//     return ((struct connection *)context)->rdma_remote_region;
// }

// char * get_peer_message_region(struct connection *conn)
// {
//   // if (s_mode == M_WRITE)
//     return conn->rdma_remote_region;
//   // else
//   //   return conn->rdma_local_region;
// }

void on_completion(struct ibv_wc *wc)
{
  struct connection *conn = (struct connection *)(uintptr_t)wc->wr_id;

  if (wc->status != IBV_WC_SUCCESS)
    die("on_completion: status is not IBV_WC_SUCCESS.");

  if (wc->opcode & IBV_WC_RECV)
  {
    conn->recv_state++;
    if (conn->recv_msg->type == MSG_MR)
    {
      memcpy(&conn->peer_mr, &conn->recv_msg->data.mr, sizeof(conn->peer_mr));
      post_receives(conn); /* only rearm for MSG_MR */

      if (conn->send_state == SS_INIT) /* received peer's MR before sending ours, so send ours back */
      {
        send_mr(conn);
      }
    }
  }
  else
  {
    conn->send_state++;
    printf("send completed successfully.\n");
  }

  if (conn->send_state == SS_MR_SENT && conn->recv_state == RS_MR_RECV)
  {

    conn->send_msg->type = MSG_DONE;
    send_message(conn);
  }
  else if (conn->send_state == SS_DONE_SENT && conn->recv_state == RS_DONE_RECV)
  {
    printf("Connected!\n");
  }
}

void on_connect(void *context)
{
  ((struct connection *)context)->connected = 1;
}

void *poll_cq(void *ctx)
{
  struct ibv_cq *cq;
  struct ibv_wc wc;
  struct connection *conn = NULL;
  struct context *self_ctx = s_ctx[cur_id];

  while (1)
  {
    TEST_NZ(ibv_get_cq_event(self_ctx->comp_channel, &cq, &ctx));
    ibv_ack_cq_events(cq, 1);
    TEST_NZ(ibv_req_notify_cq(cq, 0));

    while (ibv_poll_cq(cq, 1, &wc))
    {
      if (conn == NULL)
      {
        conn = (struct connection *)(uintptr_t)wc.wr_id;
      }

      if (conn->send_state < SS_DONE_SENT || conn->recv_state < RS_DONE_RECV)
      {
        on_completion(&wc);
      }
    }
  }

  return NULL;
}

void post_receives(struct connection *conn)
{
  struct ibv_recv_wr wr, *bad_wr = NULL;
  struct ibv_sge sge;

  wr.wr_id = (uintptr_t)conn;
  wr.next = NULL;
  wr.sg_list = &sge;
  wr.num_sge = 1;

  sge.addr = (uintptr_t)conn->recv_msg;
  sge.length = sizeof(struct message);
  sge.lkey = conn->recv_mr->lkey;

  TEST_NZ(ibv_post_recv(conn->qp, &wr, &bad_wr));
}

void register_memory(struct connection *conn)
{
  conn->send_msg = malloc(sizeof(struct message));
  conn->recv_msg = malloc(sizeof(struct message));

  // Two flag sections surround the heap region on each server.
  // the local flag is THREAD_FLAG_NUM Bytes (1MB) before the heap region,
  // and the remote flag section is 4KB after the heap region.
  size_t local_flag_section_size = THREAD_FLAG_NUM * sizeof(char);
  size_t remote_flag_section_size = 4096 * sizeof(char);
  size_t rdma_region_start_addr = remote_region_start_addr - local_flag_section_size;
  size_t rdma_region_size = local_flag_section_size + region_size + remote_flag_section_size;

  // conn->rdma_remote_region = mem_region;
  if (global_conn[0] == NULL)
  {
    conn->rdma_remote_region = mmap((void *)(rdma_region_start_addr), rdma_region_size, PROT_READ | PROT_WRITE, MAP_PRIVATE | MAP_FIXED | MAP_ANONYMOUS, -1, 0);
  }
  else
  {
    struct connection *global_connection = NULL;
    global_connection = (struct connection *)(global_conn[0]->context);
    conn->rdma_remote_region = global_connection->rdma_remote_region;
  }

  TEST_Z(conn->send_mr = ibv_reg_mr(
             s_ctx[cur_id]->pd,
             conn->send_msg,
             sizeof(struct message),
             0));

  TEST_Z(conn->recv_mr = ibv_reg_mr(
             s_ctx[cur_id]->pd,
             conn->recv_msg,
             sizeof(struct message),
             IBV_ACCESS_LOCAL_WRITE));

  // TEST_Z(conn->rdma_local_mr = ibv_reg_mr(
  //   s_ctx[cur_id]->pd,
  //   conn->rdma_local_region,
  //   RDMA_BUFFER_SIZE,
  //   ((s_mode == M_WRITE) ? 0 : IBV_ACCESS_LOCAL_WRITE)));

  TEST_Z(conn->rdma_remote_mr = ibv_reg_mr(
             s_ctx[cur_id]->pd,
             conn->rdma_remote_region,
             rdma_region_size,
             (IBV_ACCESS_LOCAL_WRITE | IBV_ACCESS_REMOTE_WRITE | IBV_ACCESS_REMOTE_READ | IBV_ACCESS_REMOTE_ATOMIC)));

  char *flag_addr = (char *)(remote_region_start_addr + region_size);
  *flag_addr = 1;
  printf("flag_addr: %p\n", flag_addr);
  printf("flag: %d\n", *flag_addr);
}

void send_message(struct connection *conn)
{
  struct ibv_send_wr wr, *bad_wr = NULL;
  struct ibv_sge sge;

  memset(&wr, 0, sizeof(wr));

  wr.wr_id = (uintptr_t)conn;
  wr.opcode = IBV_WR_SEND;
  wr.sg_list = &sge;
  wr.num_sge = 1;
  wr.send_flags = IBV_SEND_SIGNALED;

  sge.addr = (uintptr_t)conn->send_msg;
  sge.length = sizeof(struct message);
  sge.lkey = conn->send_mr->lkey;

  while (!conn->connected)
    ;

  TEST_NZ(ibv_post_send(conn->qp, &wr, &bad_wr));
}

void send_mr(void *context)
{
  struct connection *conn = (struct connection *)context;

  conn->send_msg->type = MSG_MR;
  memcpy(&conn->send_msg->data.mr, conn->rdma_remote_mr, sizeof(struct ibv_mr));

  send_message(conn);
}

void set_mode(enum mode m)
{
  s_mode = m;
}

char *drust_write_impl(size_t local_src_offset, size_t remote_dst_offset, size_t byte_size, struct rdma_cm_id *conn_id)
{
  struct connection *conn = (struct connection *)(conn_id->context);
  struct ibv_send_wr wr, *bad_wr = NULL;
  struct ibv_sge sge;
  size_t flag_offset = THREAD_FLAG_NUM * sizeof(char);

  memset(&wr, 0, sizeof(wr));

  wr.wr_id = (uintptr_t)conn;
  wr.opcode = IBV_WR_RDMA_WRITE;
  wr.sg_list = &sge;
  wr.num_sge = 1;
  wr.send_flags = IBV_SEND_SIGNALED;
  wr.wr.rdma.remote_addr = ((uintptr_t)(conn->peer_mr.addr) + flag_offset + remote_dst_offset);
  wr.wr.rdma.rkey = conn->peer_mr.rkey;

  sge.addr = (uintptr_t)(conn->rdma_remote_region) + flag_offset + local_src_offset;
  sge.length = byte_size;
  sge.lkey = conn->rdma_remote_mr->lkey;

  TEST_NZ(ibv_post_send(conn->qp, &wr, &bad_wr));
  return (char *)((uintptr_t)(conn->rdma_remote_region) + flag_offset + local_src_offset);
}

char *drust_read_impl(size_t local_dst_offset, size_t remote_src_offset, size_t byte_size, struct rdma_cm_id *conn_id, bool reading_flag)
{
  struct connection *conn = (struct connection *)(conn_id->context);
  struct ibv_send_wr wr, *bad_wr = NULL;
  struct ibv_sge sge;
  size_t flag_offset = THREAD_FLAG_NUM * sizeof(char);
  size_t actual_flag_offset = reading_flag ? 0 : flag_offset;

  memset(&wr, 0, sizeof(wr));

  wr.wr_id = (uintptr_t)conn;
  wr.opcode = IBV_WR_RDMA_READ;
  wr.sg_list = &sge;
  wr.num_sge = 1;
  wr.send_flags = IBV_SEND_SIGNALED;
  wr.wr.rdma.remote_addr = ((uintptr_t)(conn->peer_mr.addr) + flag_offset + remote_src_offset);
  wr.wr.rdma.rkey = conn->peer_mr.rkey;

  sge.addr = (uintptr_t)(conn->rdma_remote_region) + actual_flag_offset + local_dst_offset;
  sge.length = byte_size;
  sge.lkey = conn->rdma_remote_mr->lkey;

  TEST_NZ(ibv_post_send(conn->qp, &wr, &bad_wr));
  return (char *)((uintptr_t)(conn->rdma_remote_region) + actual_flag_offset + local_dst_offset);
}

/*IBV_WR_ATOMIC_CMP_AND_SWP - A 64 bits value in a remote QP's virtual space is being read,
compared with wr.atomic.compare_add and if they are equal, the value wr.atomic.swap is being written to the same memory address, in an atomic way.
No Receive Request will be consumed in the remote QP. The original data, before the compare operation, is being written to the local memory buffers specified in sg_list
*/
char *drust_xchg_impl(size_t local_src_offset, size_t remote_dst_offset, uint64_t compare_value, uint64_t swap_value, struct rdma_cm_id *conn_id)
{
  struct connection *conn = (struct connection *)(conn_id->context);
  struct ibv_send_wr wr, *bad_wr = NULL;
  struct ibv_sge sge;
  size_t flag_offset = THREAD_FLAG_NUM * sizeof(char);
  size_t actual_flag_offset = flag_offset;

  memset(&wr, 0, sizeof(wr));

  wr.wr_id = (uintptr_t)conn;
  wr.opcode = IBV_WR_ATOMIC_CMP_AND_SWP;
  wr.sg_list = &sge;
  wr.num_sge = 1;
  wr.send_flags = IBV_SEND_SIGNALED;
  wr.wr.atomic.remote_addr = ((uintptr_t)(conn->peer_mr.addr) + flag_offset + remote_dst_offset);
  wr.wr.atomic.compare_add = compare_value;
  wr.wr.atomic.swap = swap_value;
  wr.wr.atomic.rkey = conn->peer_mr.rkey;

  sge.addr = (uintptr_t)(conn->rdma_remote_region) + actual_flag_offset + local_src_offset;
  sge.length = 8;
  sge.lkey = conn->rdma_remote_mr->lkey;

  TEST_NZ(ibv_post_send(conn->qp, &wr, &bad_wr));
  return (char *)((uintptr_t)(conn->rdma_remote_region) + actual_flag_offset + local_src_offset);
}

char *drust_local_read_impl(size_t local_dst_offset, size_t remote_src_offset, size_t byte_size, struct rdma_cm_id *conn_id, bool reading_flag)
{
  struct connection *conn = (struct connection *)(conn_id->context);
  struct ibv_send_wr wr, *bad_wr = NULL;
  struct ibv_sge sge;
  size_t flag_offset = THREAD_FLAG_NUM * sizeof(char);
  size_t actual_flag_offset = reading_flag ? 0 : flag_offset;

  memset(&wr, 0, sizeof(wr));

  wr.wr_id = (uintptr_t)conn;
  wr.opcode = IBV_WR_RDMA_READ;
  wr.sg_list = &sge;
  wr.num_sge = 1;
  wr.send_flags = IBV_SEND_SIGNALED;
  wr.wr.rdma.remote_addr = ((uintptr_t)(conn->rdma_remote_region) + flag_offset + remote_src_offset);
  wr.wr.rdma.rkey = conn->rdma_remote_mr->lkey;

  sge.addr = (uintptr_t)(conn->rdma_remote_region) + actual_flag_offset + local_dst_offset;
  sge.length = byte_size;
  sge.lkey = conn->rdma_remote_mr->lkey;

  TEST_NZ(ibv_post_send(conn->qp, &wr, &bad_wr));
  return (char *)((uintptr_t)(conn->rdma_remote_region) + actual_flag_offset + local_dst_offset);
}
char *drust_xchg_local_impl(size_t local_src_offset, size_t remote_dst_offset, uint64_t compare_value, uint64_t swap_value, struct rdma_cm_id *conn_id)
{
  // printf("drust_xchg_local_impl\n");
  struct connection *conn = (struct connection *)(conn_id->context);
  struct ibv_send_wr wr, *bad_wr = NULL;
  struct ibv_sge sge;
  size_t flag_offset = THREAD_FLAG_NUM * sizeof(char);
  size_t actual_flag_offset = flag_offset;

  memset(&wr, 0, sizeof(wr));

  wr.wr_id = (uintptr_t)conn;
  wr.opcode = IBV_WR_ATOMIC_CMP_AND_SWP;
  wr.sg_list = &sge;
  wr.num_sge = 1;
  wr.send_flags = IBV_SEND_SIGNALED;
  wr.wr.atomic.remote_addr = ((uintptr_t)(conn->rdma_remote_region) + actual_flag_offset + remote_dst_offset);
  wr.wr.atomic.compare_add = compare_value;
  wr.wr.atomic.swap = swap_value;
  wr.wr.atomic.rkey = conn->rdma_remote_mr->lkey;

  sge.addr = (uintptr_t)(conn->rdma_remote_region) + actual_flag_offset + local_src_offset;
  sge.length = 8;
  sge.lkey = conn->rdma_remote_mr->lkey;

  TEST_NZ(ibv_post_send(conn->qp, &wr, &bad_wr));
  return (char *)((uintptr_t)(conn->rdma_remote_region) + actual_flag_offset + local_src_offset);
}

/* IBV_WR_ATOMIC_FETCH_AND_ADD A 64 bits value in a remote QP's virtual space is being read, added to wr.atomic.compare_add and the result is being written to the same memory address, in an atomic way. No Receive Request will be consumed in the remote QP. The original data, before the add operation, is being written to the local memory buffers specified in sg_list
 */
char *drust_fetch_add_impl(size_t local_src_offset, size_t remote_dst_offset, uint64_t add_value, struct rdma_cm_id *conn_id)
{
  struct connection *conn = (struct connection *)(conn_id->context);
  struct ibv_send_wr wr, *bad_wr = NULL;
  struct ibv_sge sge;
  size_t flag_offset = THREAD_FLAG_NUM * sizeof(char);
  size_t actual_flag_offset = flag_offset;

  memset(&wr, 0, sizeof(wr));

  wr.wr_id = (uintptr_t)conn;
  wr.opcode = IBV_WR_ATOMIC_FETCH_AND_ADD;
  wr.sg_list = &sge;
  wr.num_sge = 1;
  wr.send_flags = IBV_SEND_SIGNALED;
  wr.wr.atomic.remote_addr = ((uintptr_t)(conn->peer_mr.addr) + actual_flag_offset + remote_dst_offset);
  wr.wr.atomic.compare_add = add_value;
  wr.wr.atomic.rkey = conn->peer_mr.rkey;

  sge.addr = (uintptr_t)(conn->rdma_remote_region) + actual_flag_offset + local_src_offset;
  sge.length = 8;
  sge.lkey = conn->rdma_remote_mr->lkey;

  TEST_NZ(ibv_post_send(conn->qp, &wr, &bad_wr));
  return (char *)((uintptr_t)(conn->rdma_remote_region) + actual_flag_offset + local_src_offset);
}

void wait_until_connection_established(struct rdma_cm_id *conn_id)
{
  struct connection *conn = NULL;
  do
  {
    conn = (struct connection *)(conn_id->context);
  } while (!conn);
  while (conn->send_state < SS_DONE_SENT || conn->recv_state < RS_DONE_RECV)
    ;
  return;
}