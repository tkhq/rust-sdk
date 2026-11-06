#include <errno.h>
#include <fcntl.h>
#include <stdarg.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>
#include <unistd.h>

static int fail_config_sync(int fd, int command, ...) {
    struct stat status;
    const char *failure = getenv("TVC_TEST_SYNC_FAILURE");
    if (failure && (command == F_FULLFSYNC || command == F_BARRIERFSYNC) && fstat(fd, &status) == 0 &&
        ((strcmp(failure, "before_replacement") == 0 && S_ISREG(status.st_mode)) ||
         (strcmp(failure, "after_replacement") == 0 && S_ISDIR(status.st_mode)))) {
        errno = EIO;
        return -1;
    }
    switch (command) {
    case F_GETFD: case F_GETFL: case F_GETOWN: case F_FULLFSYNC: case F_BARRIERFSYNC:
        return fcntl(fd, command);
    default: break;
    }
    va_list args;
    va_start(args, command);
    int result;
    switch (command) {
    case F_SETFD: case F_SETFL: case F_SETOWN: case F_DUPFD: case F_DUPFD_CLOEXEC:
        result = fcntl(fd, command, va_arg(args, int)); break;
    default:
        result = fcntl(fd, command, va_arg(args, void *)); break;
    }
    va_end(args);
    return result;
}

__attribute__((used, section("__DATA,__interpose")))
static struct { const void *replacement; const void *original; } interpose = {
    (const void *)fail_config_sync, (const void *)fcntl
};
