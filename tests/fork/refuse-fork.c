/* PROTO 1154: makes fork fail with EAGAIN once the file that
 * REFUSE_FORK_FILE names exists. Linux preloads it with LD_PRELOAD, macOS
 * with DYLD_INSERT_LIBRARIES. */
#ifndef __APPLE__
#define _GNU_SOURCE
#include <dlfcn.h>
#endif
#include <errno.h>
#include <stdlib.h>
#include <sys/types.h>
#include <unistd.h>

static int refused(void) {
    const char *marker = getenv("REFUSE_FORK_FILE");
    return marker && access(marker, F_OK) == 0;
}

#ifdef __APPLE__
static pid_t refusing_fork(void) {
    if (refused()) {
        errno = EAGAIN;
        return -1;
    }
    return fork();
}

__attribute__((used)) static struct {
    const void *replacement;
    const void *replacee;
} interposers[] __attribute__((section("__DATA,__interpose"))) = {
    {(const void *)refusing_fork, (const void *)fork},
};
#else
pid_t fork(void) {
    static pid_t (*real)(void);
    if (refused()) {
        errno = EAGAIN;
        return -1;
    }
    if (!real) real = (pid_t (*)(void))dlsym(RTLD_NEXT, "fork");
    return real();
}
#endif
