#include <assert.h>
#include <stdio.h>
#include <pthread.h>
#include <unistd.h>
#include "rdma-server-lib.h"
#define ONE_MB (1024 * 1024)
#define ONE_GB (1024 * 1024 * 1024)

#define NUM_SERVERS_ (size_t)3
#define NUM_CONNECTIONS (NUM_SERVERS_ - 1)

int server_id;
const size_t heap_size = 8ull * 1024 * 1024 * 1024;
size_t heap_start;

const uint32_t kCPUFreq = 2600;

static inline uint64_t rdtscp()
{
  uint32_t a, d, c;
  asm volatile("rdtscp" : "=a"(a), "=d"(d), "=c"(c));
  return ((uint64_t)a) | (((uint64_t)d) << 32);
}

uint64_t cycles_to_us(uint64_t cycles)
{
  return cycles / kCPUFreq;
}

uint8_t conn_id_to_des(size_t conn_id)
{
  assert(conn_id < NUM_CONNECTIONS);
  if (conn_id < server_id)
  {
    return conn_id;
  }
  else
  {
    return conn_id + 1;
  }
}

void *server_helper(void *null)
{
  drust_start_server(heap_start, heap_size, server_id);
}

int test_api()
{
  pthread_t server_thread;
  int rc;
  void *status;
  int i;

  // Start server and return so we can do something else here
  rc = pthread_create(&server_thread, NULL, server_helper, NULL);
  if (rc)
  {
    printf("Error: unable to create thread\n");
    exit(-1);
  }
  // Wait until the server is ready for operations
  drust_server_ready();

  printf("server ready\n");
  scanf("%d", &i);

  // read and write here
  // No synchronization between two servers, be sure to access disjoint memory
  size_t conn_id = 0;
  while (conn_id < NUM_CONNECTIONS)
  {
    int message = NUM_CONNECTIONS * server_id + conn_id;
    size_t des = conn_id_to_des(conn_id);
    size_t remote_offset = des * heap_size + 10000 + 100 * server_id;
    size_t local_write_offset = 100 * conn_id;
    size_t local_read_offset = 100 * conn_id + 10;
    int *re = (int *)drust_write_sync(local_write_offset, remote_offset, 4, 1);
    printf("address: %lx\n", (size_t)re);
    *re = message;
    re = (int *)drust_write_sync(local_write_offset, remote_offset, 4, 1);
    printf("address: %lx\n", (size_t)re);
    re = (int *)drust_read_sync(local_read_offset, remote_offset, 4, 1);
    sleep(1);
    printf("address: %lx\n", (size_t)re);
    printf("contents: %d\n", *re);
    assert(*re == message);
    conn_id++;
  }
  // read from another server
  int *re;
  if (NUM_SERVERS_ >= 3)
  {
    if (server_id == 2)
    {
      memset((char *)(heap_start), 1, heap_size);
    }
    else if (server_id == 0)
    {
      sleep(5);
      size_t des = conn_id_to_des(1);
      printf("des: %ld\n", des);
      size_t remote_offset = des * heap_size + 0x9c88268;
      memset((char *)(heap_start + 0x7687c00), 2, 16);
      re = (int *)drust_read_sync(0x7687c78, remote_offset, 1024, 1);
      printf("contents: %d\n", *re);
    }
  }

  while (1)
  {
    sleep(1);
  }

  pthread_join(server_thread, &status);
}

int profile_read()
{
  pthread_t server_thread;
  int rc;
  void *status;
  int i;
  // Start server and return so we can do something else here
  rc = pthread_create(&server_thread, NULL, server_helper, NULL);
  if (rc)
  {
    printf("Error: unable to create thread\n");
    exit(-1);
  }
  // Wait until the server is ready for operations
  drust_server_ready();
  printf("server ready\n");
  scanf("%d", &i);

  // if (server_id == 0) {
  //   size_t remote_server_id = 1;
  //   size_t remote_offset = remote_server_id * heap_size + 10000 + 100 * server_id;
  //   size_t local_write_offset = 1024;
  //   size_t local_read_offset = 1024 + 1024;

  //   int* re = (int*)drust_write_sync(local_write_offset, remote_offset, 512, 1);
  //   printf("address: %lx\n", (size_t)re);
  //   for (int i = 0; i < 512/4; i++) {
  //     re[i] = i;
  //   }
  //   re = (int*)drust_write_sync(local_write_offset, remote_offset, 512, 1);
  //   printf("address: %lx\n", (size_t)re);

  //   size_t tim = 0;

  //   for(int times = 0; times < 100000; times++) {
  //     uint64_t start = rdtscp();
  //     re = (int*)drust_read_sync(local_read_offset, remote_offset, 512, 1);
  //     uint64_t end = rdtscp();
  //     printf("address: %lx\n", (size_t)re);
  //     for (int i = 0; i < 512/4; i++) {
  //       if (re[i] != i) {
  //         printf("error: %d, %d\n", i, re[i]);
  //       }
  //       re[i] = 0;
  //     }
  //     tim += end - start;
  //     printf("cycles: %ld\n", end - start);
  //     printf("time: %ld\n", cycles_to_us(end - start));
  //   }
  //   printf("average cycles: %ld\n", tim/100000);
  //   printf("average time: %ld\n", cycles_to_us(tim/100000));
  // }

  if (server_id == 1)
  {
    size_t remote_server_id = 0;
    size_t remote_offset = remote_server_id * heap_size + 10000 + 100 * server_id;
    size_t local_write_offset = 1024;
    size_t local_read_offset = 1024 + 1024;

    int *re = (int *)drust_write_sync(local_write_offset, remote_offset, 512, 1);
    printf("address: %lx\n", (size_t)re);
    for (int i = 0; i < 512 / 4; i++)
    {
      re[i] = i;
    }
    re = (int *)drust_write_sync(local_write_offset, remote_offset, 512, 1);
    printf("address: %lx\n", (size_t)re);

    size_t tim = 0;

    for (int times = 0; times < 100000; times++)
    {
      uint64_t start = rdtscp();
      re = (int *)drust_read_sync(local_read_offset, remote_offset, 512, 1);
      uint64_t end = rdtscp();
      // printf("address: %lx\n", (size_t)re);
      for (int i = 0; i < 512 / 4; i++)
      {
        if (re[i] != i)
        {
          printf("error: %d, %d\n", i, re[i]);
        }
        re[i] = 0;
      }
      tim += end - start;
      // printf("cycles: %ld\n", end - start);
      // printf("time: %ld\n", cycles_to_us(end - start));
    }
    printf("average cycles: %ld\n", tim / 100000);
    printf("average time: %ld\n", cycles_to_us(tim / 100000));
  }

  while (1)
  {
    sleep(1);
  }
  pthread_join(server_thread, &status);
}

int main()
{
  scanf("%d", &server_id);
  heap_start = (size_t)0x400000000000ull + heap_size * server_id;
  profile_read();
}
