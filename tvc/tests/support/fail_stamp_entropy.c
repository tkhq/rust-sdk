#include <dlfcn.h>
#include <errno.h>
#include <execinfo.h>
#include <stdlib.h>
#include <string.h>
#include <sys/random.h>
#include <unistd.h>

static int from_fresh_stamp(void) {
    void *frames[64];
    int count = backtrace(frames, 64);
    for (int index = 0; index < count; index++) {
        Dl_info symbol;
        if (dladdr(frames[index], &symbol) && symbol.dli_sname &&
            (strstr(symbol.dli_sname, "stamp_fresh") ||
             strstr(symbol.dli_sname, "stamp_with_entropy"))) {
            return 1;
        }
    }
    return 0;
}
static int fail_getentropy(void *buffer, size_t length) {
    const char *marker = getenv("TVC572_ENTROPY_FAILURE_MARKER");
    if (length == 32 && marker && access(marker, F_OK) == 0 && from_fresh_stamp()) {
        errno = EIO;
        return -1;
    }
    return getentropy(buffer, length);
}
__attribute__((used, section("__DATA,__interpose")))
static struct { const void *replacement; const void *original; } interpose = {
    (const void *)fail_getentropy, (const void *)getentropy
};
