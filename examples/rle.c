#include <stdint.h>
#include <sys/types.h>
#include <sys/stat.h>
#include <fcntl.h>
#include <stdio.h>
#include <stdlib.h>
uint8_t buf[4*1024*1024];

void error_exit(const char *msg) {
    fprintf(stderr, "assert failed: %s\n", msg);
    exit(1);
}
#define A(it) {if (!(it)) error_exit(#it);}

void main(int argc, char **argv) {
    int fd,nb;
    A( argc == 2 );
    A( (fd = open(argv[1], O_RDONLY)) >= 0 );
    for(;;) {
        A( (nb = read(fd, buf, sizeof(buf)))> 0 );
        fprintf(stderr, "%d ", nb);
    }
}
