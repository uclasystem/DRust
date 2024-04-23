#include "rdma-server-lib.h"
#include "rdma-common.h"

const int TIMEOUT_IN_MS = 500; /* ms */

static int on_connect_request(struct rdma_cm_id *id);
static int on_connection(struct rdma_cm_id *id, bool active);
static int on_disconnect(struct rdma_cm_id *id);
static int on_event_active(struct rdma_cm_event *event);
static int on_event_passive(struct rdma_cm_event *event);
static void init_mem_region();
static void make_active_connection(const char *passive_ip_str, const char *port_str);
static void make_passive_connection(const char *passive_ip_str, const char *port_str);
static void usage(const char *argv0);

static size_t global_server_id = 0;

int drust_start_server(size_t heap_start, size_t heap_size, size_t server_id)
{
  if (server_id > NUM_SERVERS)
  {
    printf("server_id should be less than NUM_SERVERS\n");
    printf("server_id: %ld\n", server_id);
    exit(1);
  }
  remote_region_start_addr = heap_start;
  region_size = heap_size;
  cur_id = 0;

  set_mode(M_WRITE);

  global_server_id = server_id;

  const char *ip_str[9] = {"10.0.0.1", "10.0.0.2", "10.0.0.3", "10.0.0.4", "10.0.0.5", "10.0.0.6", "10.0.0.10", "10.0.0.11", "10.0.0.1"};
  const char *port_str[8] = {"9400", "9401", "9402", "9403", "9404", "9405", "9406", "9407"};
  printf("%s, trying to bind to %s:%s.\n", __func__, ip_str[server_id], port_str[cur_id]);

  // init_mem_region();
  // For every server, with a given `server_id` i and `NUM_SERVERS` n
  // It needs to first make i passive connections, one after the other and then (n - 1 - i) active connections

  // Make the passive connections
  while (cur_id < server_id)
  {
    make_passive_connection(ip_str[server_id], port_str[cur_id]);
    wait_until_connection_established(global_conn[cur_id]);
    cur_id++;
  }

  size_t passive_ip_idx = NUM_SERVERS;
  // global_conn[server_id:NUM_SERVERS] are in reverse order of the servers
  // This is fixed during drust_server_ready
  while (cur_id < NUM_SERVERS)
  {
    // Connect to the servers backwards, starting from server[NUM_SERVERS-1]
    printf("passive_ip_idx: %lu\n", passive_ip_idx);
    make_active_connection(ip_str[passive_ip_idx], port_str[server_id]);
    wait_until_connection_established(global_conn[cur_id]);
    cur_id++;
    passive_ip_idx--;
  }

  // make_passive_connection(ip_str[server_id], port_str[cur_id]);
  // wait_until_connection_established(global_conn[cur_id]);
  // cur_id++;

  // TODO: listen for signals and properly stop connections
  while (1)
  {
    sleep(1);
  }
  // rdma_destroy_id(listener);
  // rdma_destroy_event_channel(ec);

  return 0;
}

void init_mem_region()
{
  size_t local_flag_section_size = THREAD_FLAG_NUM * sizeof(char);
  size_t remote_flag_section_size = 4096 * sizeof(char);
  size_t rdma_region_start_addr = remote_region_start_addr - local_flag_section_size;
  size_t rdma_region_size = local_flag_section_size + region_size + remote_flag_section_size;
  printf("rdma_region_start_addr: %lx\n", rdma_region_start_addr);
  printf("rdma_region_size: %lx\n", rdma_region_size);
  mem_region = (char *)mmap((void *)(rdma_region_start_addr), rdma_region_size, PROT_READ | PROT_WRITE, MAP_PRIVATE | MAP_FIXED | MAP_ANONYMOUS, -1, 0);
  // parallelly touch this mem region
  for (size_t i = 0; i < rdma_region_size; i += 4096)
  {
    mem_region[i] = 0;
  }
}

void make_active_connection(const char *passive_ip_str, const char *port_str)
{
  struct addrinfo *addr;
  struct rdma_cm_event *event = NULL;
  struct rdma_cm_id *conn = NULL;
  struct rdma_event_channel *ec = NULL;
  TEST_NZ(getaddrinfo(passive_ip_str, port_str, NULL, &addr));
  TEST_Z(ec = rdma_create_event_channel());
  TEST_NZ(rdma_create_id(ec, &conn, NULL, RDMA_PS_TCP));
  TEST_NZ(rdma_resolve_addr(conn, NULL, addr->ai_addr, TIMEOUT_IN_MS));
  freeaddrinfo(addr);

  while (rdma_get_cm_event(ec, &event) == 0)
  {
    struct rdma_cm_event event_copy;

    memcpy(&event_copy, event, sizeof(*event));
    rdma_ack_cm_event(event);

    if (on_event_active(&event_copy))
      break;
  }
}

void make_passive_connection(const char *passive_ip_str, const char *port_str)
{
  struct sockaddr_in6 addr;
  struct rdma_cm_event *event = NULL;
  struct rdma_cm_id *listener = NULL;
  struct rdma_event_channel *ec = NULL;
  uint16_t port = 0;
  memset(&addr, 0, sizeof(addr));
  addr.sin6_family = AF_INET6;
  inet_pton(AF_INET6, passive_ip_str, &addr.sin6_addr);
  addr.sin6_port = htons(atoi(port_str));
  inet_pton(AF_INET6, passive_ip_str, &addr.sin6_addr);

  TEST_Z(ec = rdma_create_event_channel());
  TEST_NZ(rdma_create_id(ec, &listener, NULL, RDMA_PS_TCP));
  TEST_NZ(rdma_bind_addr(listener, (struct sockaddr *)&addr));
  TEST_NZ(rdma_listen(listener, 10)); /* backlog=10 is arbitrary */
  port = ntohs(rdma_get_src_port(listener));
  printf("listening on port %d.\n", port);

  while (rdma_get_cm_event(ec, &event) == 0)
  {
    struct rdma_cm_event event_copy;

    memcpy(&event_copy, event, sizeof(*event));
    rdma_ack_cm_event(event);

    if (on_event_passive(&event_copy))
      break;
  }
}

void drust_server_ready()
{
  for (int i = 0; i < NUM_SERVERS; ++i)
  {
    while (!global_conn[i])
      ;
    wait_until_connection_established(global_conn[i]);
  }
  // Reverse the order of global_conn[server_id:NUM_SERVERS]
  // Here all global_conn elements are already assigned in build_connection,
  // but not yet accessed by drust_read or _write.
  // So they are safe to be moved.
  int i = global_server_id, j = NUM_SERVERS - 1;
  while (i < j)
  {
    struct rdma_cm_id *tmp = global_conn[i];
    global_conn[i] = global_conn[j];
    global_conn[j] = tmp;
    i++;
    j--;
  }
  printf("All servers are ready\n");
}

int on_addr_resolved(struct rdma_cm_id *id)
{
  printf("address resolved.\n");

  build_connection(id);
  TEST_NZ(rdma_resolve_route(id, TIMEOUT_IN_MS));

  return 0;
}

int on_route_resolved(struct rdma_cm_id *id)
{
  struct rdma_conn_param cm_params;

  printf("route resolved.\n");
  build_params(&cm_params);
  TEST_NZ(rdma_connect(id, &cm_params));
  printf("connect request sent.\n");

  return 0;
}

int on_connect_request(struct rdma_cm_id *id)
{
  struct rdma_conn_param cm_params;

  printf("received connection request.\n");
  build_connection(id);
  build_params(&cm_params);
  TEST_NZ(rdma_accept(id, &cm_params));
  printf("on_connect_request\n");
  fflush(stdout);

  return 0;
}

int on_connection(struct rdma_cm_id *id, bool active)
{
  on_connect(id->context);
  printf("on_connection\n");
  fflush(stdout);

  if (active)
  {
    printf("%s: Sending MR to the passive side\n", __func__);
    send_mr(id->context);
  }

  return 0;
}

int on_disconnect(struct rdma_cm_id *id)
{
  printf("peer disconnected.\n");

  destroy_connection(id->context);
  return 0;
}

int on_event_active(struct rdma_cm_event *event)
{
  int r = 0;

  if (event->event == RDMA_CM_EVENT_ADDR_RESOLVED)
    r = on_addr_resolved(event->id);
  else if (event->event == RDMA_CM_EVENT_ROUTE_RESOLVED)
    r = on_route_resolved(event->id);
  else if (event->event == RDMA_CM_EVENT_ESTABLISHED)
  {
    on_connection(event->id, true);
    r = 1;
  }
  else if (event->event == RDMA_CM_EVENT_DISCONNECTED)
    r = on_disconnect(event->id);
  else
  {
    fprintf(stderr, "on_event_active event ID: %d\n", event->event);
    die("on_event_active: unknown event.");
  }

  return r;
}

int on_event_passive(struct rdma_cm_event *event)
{
  int r = 0;

  if (event->event == RDMA_CM_EVENT_CONNECT_REQUEST)
    r = on_connect_request(event->id);
  else if (event->event == RDMA_CM_EVENT_ESTABLISHED)
  {
    // Return on establishing the connection, so the server
    // can work on the connection of another server.
    on_connection(event->id, false);
    r = 1;
  }
  else if (event->event == RDMA_CM_EVENT_DISCONNECTED)
    r = on_disconnect(event->id);
  else
  {
    printf("on_event_active event ID: %d\n", event->event);
    die("on_event_passive: unknown event.");
  }

  return r;
}

void usage(const char *argv0)
{
  fprintf(stderr, "usage: %s <mode>\n  mode = \"read\", \"write\"\n", argv0);
  exit(1);
}

// Mapping from destination server ID to connection ID is not an identity function.
// E.g., for 3 servers, the mapping is:
//      des  0  1  2
// src conn
// 0         X  0  1
// 1         0  X  1
// 2         0  1  X
// Where X means illegal (accessing the source itself)
size_t des_to_conn_id(size_t des)
{
  if (des == global_server_id)
  {
    die("des_to_conn_id: Illegal destination: same as local ID\n");
    return 0;
  }
  else if (des < global_server_id)
  {
    return des;
  }
  else
  {
    return des - 1;
  }
}

void offset_to_conn_id_and_conn_offset(size_t offset, size_t *conn_id, size_t *conn_offset)
{
  size_t remote_server_id = offset / region_size;
  *conn_id = des_to_conn_id(remote_server_id);
  *conn_offset = offset % region_size;
}

bool remote_is_driver(size_t conn_id)
{
  return conn_id == 0;
}

void rdma_sync(size_t conn_id, size_t thread_flag_id)
{
  // printf("rdma_sync\n");
  // printf("thread_flag_id: %ld\n", thread_flag_id);
  // printf("conn_id: %ld\n", conn_id);
  if (thread_flag_id >= THREAD_FLAG_NUM)
  {
    printf("thread_flag_id should be less than THREAD_FLAG_NUM\n");
    printf("thread_flag_id: %ld\n", thread_flag_id);
    exit(1);
  }
  char *flag_addr = (char *)(remote_region_start_addr - (THREAD_FLAG_NUM - thread_flag_id) * sizeof(char));
  *flag_addr = 0;
  size_t remote_flag_offset = region_size;
  volatile char *flag_addr_v = drust_read_impl(thread_flag_id * sizeof(char), remote_flag_offset, 1, global_conn[conn_id], true);
  while (*flag_addr_v == 0)
  {
    // usleep(1);
  }
}

// dst_offset includes the offset of the CPU server (i.e., driver) region
size_t drust_write(size_t local_src_offset, size_t dst_offset, size_t byte_size)
{
  size_t conn_id, conn_offset;
  offset_to_conn_id_and_conn_offset(dst_offset, &conn_id, &conn_offset);
  char *addr = drust_write_impl(local_src_offset, conn_offset, byte_size, global_conn[conn_id]);
  return (size_t)addr;
}

size_t drust_write_sync(size_t local_src_offset, size_t dst_offset, size_t byte_size, size_t thread_flag_id)
{
  size_t conn_id, conn_offset;
  offset_to_conn_id_and_conn_offset(dst_offset, &conn_id, &conn_offset);
  char *addr = drust_write_impl(local_src_offset, conn_offset, byte_size, global_conn[conn_id]);
  rdma_sync(conn_id, thread_flag_id);
  return (size_t)addr;
}

// src_offset includes the offset of the CPU server (i.e., driver) region
size_t drust_read(size_t local_dst_offset, size_t src_offset, size_t byte_size)
{
  size_t conn_id, conn_offset;
  offset_to_conn_id_and_conn_offset(src_offset, &conn_id, &conn_offset);
  char *addr = drust_read_impl(local_dst_offset, conn_offset, byte_size, global_conn[conn_id], false);
  return (size_t)addr;
}

size_t drust_read_sync(size_t local_dst_offset, size_t src_offset, size_t byte_size, size_t thread_flag_id)
{
  size_t conn_id, conn_offset;
  offset_to_conn_id_and_conn_offset(src_offset, &conn_id, &conn_offset);
  // printf("local_dst_offset: %lx src_offset: %lx byte_size: %lx conn_id: %ld conn_offset: %lx\n", local_dst_offset, src_offset, byte_size, conn_id, conn_offset);
  char *addr = drust_read_impl(local_dst_offset, conn_offset, byte_size, global_conn[conn_id], false);
  rdma_sync(conn_id, thread_flag_id);
  return (size_t)addr;
}

size_t drust_atomic_cmp_exchg(size_t local_src_offset, size_t remote_dst_offset, size_t old_value, size_t new_value)
{
  size_t conn_id, conn_offset;
  offset_to_conn_id_and_conn_offset(remote_dst_offset, &conn_id, &conn_offset);
  char *addr = drust_xchg_impl(local_src_offset, conn_offset, old_value, new_value, global_conn[conn_id]);
  return (size_t)addr;
}

size_t drust_atomic_cmp_exchg_sync(size_t local_src_offset, size_t remote_dst_offset, size_t old_value, size_t new_value, size_t thread_flag_id)
{
  size_t conn_id, conn_offset;
  offset_to_conn_id_and_conn_offset(remote_dst_offset, &conn_id, &conn_offset);
  char *addr = drust_xchg_impl(local_src_offset, conn_offset, old_value, new_value, global_conn[conn_id]);
  // printf("drust_atomic_cmp_exchg_sync, returned addr: %lx, returned old value: %lx\n", (size_t)addr, *(size_t*)addr);
  rdma_sync(conn_id, thread_flag_id);
  return (size_t)addr;
}

size_t drust_local_atomic_cmp_exchg_sync(size_t local_src_offset, size_t remote_dst_offset, size_t old_value, size_t new_value, size_t thread_flag_id)
{
  size_t conn_id, conn_offset;
  conn_id = 0;
  conn_offset = remote_dst_offset % region_size;
  char *addr = drust_xchg_local_impl(local_src_offset, conn_offset, old_value, new_value, global_conn[conn_id]);
  // printf("drust_atomic_cmp_exchg_sync, returned addr: %lx, returned old value: %lx\n", (size_t)addr, *(size_t*)addr);
  // if(thread_flag_id >= THREAD_FLAG_NUM) {
  //   printf("thread_flag_id should be less than THREAD_FLAG_NUM\n");
  //   printf("thread_flag_id: %ld\n", thread_flag_id);
  //   exit(1);
  // }
  // char* flag_addr = (char*)(remote_region_start_addr - (THREAD_FLAG_NUM - thread_flag_id) * sizeof(char));
  // *flag_addr = 0;
  // size_t remote_flag_offset = region_size;
  // volatile char* flag_addr_v = drust_local_read_impl(thread_flag_id * sizeof(char), remote_flag_offset, 1, global_conn[conn_id], true);
  // while (*flag_addr_v == 0) {
  // }

  return (size_t)addr;
}

size_t drust_atomic_fetch_add(size_t local_src_offset, size_t remote_dst_offset, size_t add_value)
{
  size_t conn_id, conn_offset;
  offset_to_conn_id_and_conn_offset(remote_dst_offset, &conn_id, &conn_offset);
  char *addr = drust_fetch_add_impl(local_src_offset, conn_offset, add_value, global_conn[conn_id]);
  return (size_t)addr;
}

size_t drust_atomic_fetch_add_sync(size_t local_src_offset, size_t remote_dst_offset, size_t add_value, size_t thread_flag_id)
{
  size_t conn_id, conn_offset;
  offset_to_conn_id_and_conn_offset(remote_dst_offset, &conn_id, &conn_offset);
  char *addr = drust_fetch_add_impl(local_src_offset, conn_offset, add_value, global_conn[conn_id]);
  rdma_sync(conn_id, thread_flag_id);
  return (size_t)addr;
}

int copy_mem(size_t src_addr, size_t dst_addr, size_t byte_size)
{
  memcpy((char *)dst_addr, (char *)src_addr, byte_size);
  return 1;
}

size_t register_mem(size_t heap_start, size_t heap_size)
{
  size_t re = (size_t)mmap((char *)heap_start, heap_size, PROT_READ | PROT_WRITE, MAP_PRIVATE | MAP_FIXED | MAP_ANONYMOUS, -1, 0);
  memset((char *)re, 0, heap_size);
  return re;
}