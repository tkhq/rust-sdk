#include <errno.h>
#include <fcntl.h>
#include <stdlib.h>
#include <sys/socket.h>
#include <unistd.h>

static int deny_connect(int socket, const struct sockaddr *address, socklen_t length) {
    (void)socket;
    (void)address;
    (void)length;
    const char *marker = getenv("TVC572_CONNECT_MARKER");
    if (marker) {
        int file = open(marker, O_WRONLY | O_CREAT | O_APPEND, 0600);
        if (file >= 0) {
            write(file, "connect\n", 8);
            close(file);
        }
    }
    errno = ECONNREFUSED;
    return -1;
}
__attribute__((used, section("__DATA,__interpose")))
static struct { const void *replacement; const void *original; } interpose = {
    (const void *)deny_connect, (const void *)connect
};
