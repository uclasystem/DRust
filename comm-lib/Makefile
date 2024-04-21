.PHONY: clean

CFLAGS  := -Wall -Werror -g -Wno-unused-function
LD      := gcc
LDLIBS  := ${LDLIBS} -lrdmacm -libverbs -lpthread
AFLAGS	:= rcs
LINKLIBS:= ${LDLIBS} -lmyrdma

APPS    := main
LIBS    := libmyrdma.a

all: ${APPS}
lib: ${LIBS}

# rdma-server: rdma-common.o rdma-server.o
# 	${LD} -o $@ $^ ${LDLIBS}

libmyrdma.a: rdma-common.o rdma-server-lib.o
	ar ${AFLAGS} $@ $^
	ranlib $@

main: main.c libmyrdma.a
	${LD} -o $@ $^ -L. ${LINKLIBS}

clean:
	rm -f *.o ${APPS}
	rm -f *.a ${LIBS}

