#include <errno.h>
#include <stdlib.h>
#include <unistd.h>
#include <sys/random.h>

static int armed(size_t length) {
    const char *marker = getenv("TVC572_ENTROPY_FAILURE_MARKER");
    return length == 32 && marker && access(marker, F_OK) == 0;
}

static int fail_getentropy(void *buffer, size_t length) {
    if (armed(length)) {
        errno = EIO;
        return -1;
    }
    return getentropy(buffer, length);
}
__attribute__((used, section("__DATA,__interpose")))
static struct { const void *replacement; const void *original; } interpose = {
    (const void *)fail_getentropy, (const void *)getentropy
};
