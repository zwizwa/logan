// simple test that reads and walks the data.
// C isn't fast enough

#include <sys/types.h>
#include <sys/stat.h>
#include <fcntl.h>
#include <stdio.h>
#include <stdlib.h>

unsigned int buf[1*1024*1024];

void error_exit(const char *msg) {
    fprintf(stderr, "assert failed: %s\n", msg);
    exit(1);
}
#define A(it) {if (!(it)) error_exit(#it);}

void main(int argc, char **argv) {
    int fd,nb,i,acc=0;
    A( argc == 2 );
    A( (fd = open(argv[1], O_RDONLY)) >= 0 );
    for(;;) {
        A( (nb = read(fd, buf, sizeof(buf))) > 0 );
        for(i=0;i<nb/4;i++) { acc += buf[i]; }
        fprintf(stderr, "%d ", acc);
    }
}
