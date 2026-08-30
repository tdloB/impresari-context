// SPDX-License-Identifier: Apache-2.0
#define _GNU_SOURCE

#include <errno.h>
#include <fcntl.h>
#include <linux/sched.h>
#include <signal.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/resource.h>
#include <sys/syscall.h>
#include <sys/wait.h>
#include <unistd.h>

#ifndef CLONE_INTO_CGROUP
#define CLONE_INTO_CGROUP 0x200000000ULL
#endif
#ifndef SYS_clone3
#define SYS_clone3 435
#endif

enum worker_mode {
    WORKER_OK,
    WORKER_PIDS,
    WORKER_MEMORY,
    WORKER_CPU,
    WORKER_SLEEP,
    WORKER_FLOOD,
    WORKER_CRASH,
};

static enum worker_mode parse_mode(const char *const value) {
    if (strcmp(value, "ok") == 0) return WORKER_OK;
    if (strcmp(value, "pids") == 0) return WORKER_PIDS;
    if (strcmp(value, "memory") == 0) return WORKER_MEMORY;
    if (strcmp(value, "cpu") == 0) return WORKER_CPU;
    if (strcmp(value, "sleep") == 0) return WORKER_SLEEP;
    if (strcmp(value, "flood") == 0) return WORKER_FLOOD;
    if (strcmp(value, "crash") == 0) return WORKER_CRASH;
    exit(20);
}

static void write_exact(const char *const bytes, const size_t length) {
    if (write(STDOUT_FILENO, bytes, length) != (ssize_t)length) _exit(26);
}

static void run_worker(const enum worker_mode mode) {
    if (mode == WORKER_OK) {
        write_exact("ok\n", 3);
        _exit(0);
    }
    if (mode == WORKER_PIDS) {
        errno = 0;
        const pid_t child = fork();
        const bool denied = child == -1 && errno == EAGAIN;
        if (child == 0) _exit(90);
        write_exact(denied ? "denied\n" : "allowed\n", denied ? 7U : 8U);
        _exit(denied ? 0 : 21);
    }
    if (mode == WORKER_MEMORY) {
        const size_t length = 256U * 1024U * 1024U;
        volatile unsigned char *const bytes = malloc(length);
        if (bytes == NULL) _exit(22);
        for (size_t offset = 0; offset < length; offset += 4096U) {
            bytes[offset] = (unsigned char)(offset & 0xffU);
        }
        _exit(23);
    }
    if (mode == WORKER_CPU) {
        const struct rlimit limit = {.rlim_cur = 1, .rlim_max = 1};
        if (setrlimit(RLIMIT_CPU, &limit) != 0) _exit(24);
        volatile uint64_t counter = 0;
        for (;;) ++counter;
    }
    if (mode == WORKER_SLEEP) {
        for (;;) (void)pause();
    }
    if (mode == WORKER_FLOOD) {
        unsigned char block[4096];
        memset(block, 'x', sizeof(block));
        for (;;) {
            if (write(STDOUT_FILENO, block, sizeof(block)) <= 0) _exit(0);
        }
    }
    if (mode == WORKER_CRASH) abort();
    _exit(25);
}

static int run_in_cgroup(const char *const cgroup,
                         const enum worker_mode mode) {
    const int cgroup_fd = open(cgroup, O_RDONLY | O_DIRECTORY | O_CLOEXEC);
    if (cgroup_fd < 0) return 30;
    const struct clone_args arguments = {
        .flags = CLONE_INTO_CGROUP,
        .exit_signal = SIGCHLD,
        .cgroup = (uint64_t)cgroup_fd,
    };
    const pid_t pid = (pid_t)syscall(SYS_clone3, &arguments, sizeof(arguments));
    if (pid < 0) {
        (void)close(cgroup_fd);
        return errno == ENOSYS || errno == EINVAL || errno == EPERM ? 31 : 32;
    }
    if (pid == 0) {
        (void)close(cgroup_fd);
        run_worker(mode);
    }
    (void)close(cgroup_fd);
    int status = 0;
    while (waitpid(pid, &status, 0) < 0) {
        if (errno != EINTR) return 33;
    }
    if (mode == WORKER_MEMORY) {
        return WIFSIGNALED(status) && WTERMSIG(status) == SIGKILL ? 0 : 34;
    }
    if (mode == WORKER_CPU) {
        return WIFSIGNALED(status) &&
                       (WTERMSIG(status) == SIGKILL ||
                        WTERMSIG(status) == SIGXCPU)
                   ? 0
                   : 35;
    }
    if (mode == WORKER_SLEEP) {
        return WIFSIGNALED(status) && WTERMSIG(status) == SIGKILL ? 0 : 36;
    }
    if (mode == WORKER_CRASH) {
        return WIFSIGNALED(status) && WTERMSIG(status) == SIGABRT ? 0 : 37;
    }
    return WIFEXITED(status) ? WEXITSTATUS(status) : 38;
}

int main(const int argument_count, char *const arguments[]) {
    if (argument_count != 4 || strcmp(arguments[1], "run") != 0) return 2;
    return run_in_cgroup(arguments[2], parse_mode(arguments[3]));
}
