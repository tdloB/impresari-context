// SPDX-License-Identifier: Apache-2.0
#define _GNU_SOURCE

#include <errno.h>
#include <fcntl.h>
#include <linux/audit.h>
#include <linux/filter.h>
#include <linux/landlock.h>
#include <linux/sched.h>
#include <linux/seccomp.h>
#include <signal.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/prctl.h>
#include <sys/resource.h>
#include <sys/socket.h>
#include <sys/stat.h>
#include <sys/syscall.h>
#include <sys/wait.h>
#include <unistd.h>

#ifndef SYS_landlock_create_ruleset
#define SYS_landlock_create_ruleset 444
#endif
#ifndef SYS_landlock_add_rule
#define SYS_landlock_add_rule 445
#endif
#ifndef SYS_landlock_restrict_self
#define SYS_landlock_restrict_self 446
#endif
#ifndef SYS_clone3
#define SYS_clone3 435
#endif
#ifndef CLONE_INTO_CGROUP
#define CLONE_INTO_CGROUP 0x200000000ULL
#endif

#if defined(__x86_64__)
#define COMPAT_AUDIT_ARCH AUDIT_ARCH_X86_64
#else
#error "ADR-0099 launcher is admitted only for x86_64 Linux"
#endif

#define EXPECTED_THREAD_CLONE_FLAGS 0x003d0f00U
static int landlock_create_ruleset(const struct landlock_ruleset_attr *attr,
                                   size_t size, uint32_t flags) {
    return (int)syscall(SYS_landlock_create_ruleset, attr, size, flags);
}

static int landlock_add_rule(int ruleset_fd,
                             enum landlock_rule_type rule_type,
                             const void *rule_attr, uint32_t flags) {
    return (int)syscall(SYS_landlock_add_rule, ruleset_fd, rule_type,
                        rule_attr, flags);
}

static int landlock_restrict_self(int ruleset_fd, uint32_t flags) {
    return (int)syscall(SYS_landlock_restrict_self, ruleset_fd, flags);
}

static uint64_t handled_filesystem_access(int abi) {
    uint64_t access = LANDLOCK_ACCESS_FS_EXECUTE |
                      LANDLOCK_ACCESS_FS_WRITE_FILE |
                      LANDLOCK_ACCESS_FS_READ_FILE |
                      LANDLOCK_ACCESS_FS_READ_DIR |
                      LANDLOCK_ACCESS_FS_REMOVE_DIR |
                      LANDLOCK_ACCESS_FS_REMOVE_FILE |
                      LANDLOCK_ACCESS_FS_MAKE_CHAR |
                      LANDLOCK_ACCESS_FS_MAKE_DIR |
                      LANDLOCK_ACCESS_FS_MAKE_REG |
                      LANDLOCK_ACCESS_FS_MAKE_SOCK |
                      LANDLOCK_ACCESS_FS_MAKE_FIFO |
                      LANDLOCK_ACCESS_FS_MAKE_BLOCK |
                      LANDLOCK_ACCESS_FS_MAKE_SYM;
#ifdef LANDLOCK_ACCESS_FS_REFER
    if (abi >= 2) {
        access |= LANDLOCK_ACCESS_FS_REFER;
    }
#endif
#ifdef LANDLOCK_ACCESS_FS_TRUNCATE
    if (abi >= 3) {
        access |= LANDLOCK_ACCESS_FS_TRUNCATE;
    }
#endif
#ifdef LANDLOCK_ACCESS_FS_IOCTL_DEV
    if (abi >= 5) {
        access |= LANDLOCK_ACCESS_FS_IOCTL_DEV;
    }
#endif
    return access;
}

static bool install_read_execute_landlock(const char *job_root) {
    const int abi = landlock_create_ruleset(NULL, 0,
                                            LANDLOCK_CREATE_RULESET_VERSION);
    if (abi < 1) {
        return false;
    }
    const struct landlock_ruleset_attr ruleset = {
        .handled_access_fs = handled_filesystem_access(abi),
    };
    const int ruleset_fd =
        landlock_create_ruleset(&ruleset, sizeof(ruleset), 0);
    if (ruleset_fd < 0) {
        return false;
    }
    const int root_fd = open(job_root, O_PATH | O_CLOEXEC | O_DIRECTORY);
    if (root_fd < 0) {
        (void)close(ruleset_fd);
        return false;
    }
    const struct landlock_path_beneath_attr rule = {
        .allowed_access = LANDLOCK_ACCESS_FS_EXECUTE |
                          LANDLOCK_ACCESS_FS_READ_FILE |
                          LANDLOCK_ACCESS_FS_READ_DIR,
        .parent_fd = root_fd,
    };
    const bool added = landlock_add_rule(
                           ruleset_fd, LANDLOCK_RULE_PATH_BENEATH, &rule, 0) ==
                       0;
    const bool restricted = added && landlock_restrict_self(ruleset_fd, 0) == 0;
    (void)close(root_fd);
    (void)close(ruleset_fd);
    return restricted;
}

static void close_unrelated_descriptors(void) {
#ifdef SYS_close_range
    if (syscall(SYS_close_range, 3U, ~0U, 0U) == 0) {
        return;
    }
#endif
    const long maximum = sysconf(_SC_OPEN_MAX);
    const int upper = maximum > 0 && maximum < 1048576 ? (int)maximum : 65536;
    for (int fd = 3; fd < upper; ++fd) {
        (void)close(fd);
    }
}

#define ALLOW_SYSCALL(name)                                                   \
    BPF_JUMP(BPF_JMP | BPF_JEQ | BPF_K, SYS_##name, 0, 1),                  \
        BPF_STMT(BPF_RET | BPF_K, SECCOMP_RET_ALLOW)

static bool install_seccomp(void) {
    struct sock_filter filter[] = {
        BPF_STMT(BPF_LD | BPF_W | BPF_ABS,
                 (uint32_t)offsetof(struct seccomp_data, arch)),
        BPF_JUMP(BPF_JMP | BPF_JEQ | BPF_K, COMPAT_AUDIT_ARCH, 1, 0),
        BPF_STMT(BPF_RET | BPF_K, SECCOMP_RET_KILL_PROCESS),
        BPF_STMT(BPF_LD | BPF_W | BPF_ABS,
                 (uint32_t)offsetof(struct seccomp_data, nr)),
        BPF_JUMP(BPF_JMP | BPF_JEQ | BPF_K, SYS_clone, 0, 6),
        BPF_STMT(BPF_LD | BPF_W | BPF_ABS,
                 (uint32_t)offsetof(struct seccomp_data, args[0])),
        BPF_JUMP(BPF_JMP | BPF_JEQ | BPF_K, EXPECTED_THREAD_CLONE_FLAGS, 0, 3),
        BPF_STMT(BPF_LD | BPF_W | BPF_ABS,
                 (uint32_t)offsetof(struct seccomp_data, args[0]) + 4U),
        BPF_JUMP(BPF_JMP | BPF_JEQ | BPF_K, 0, 0, 1),
        BPF_STMT(BPF_RET | BPF_K, SECCOMP_RET_ALLOW),
        BPF_STMT(BPF_RET | BPF_K,
                 SECCOMP_RET_ERRNO | (EPERM & SECCOMP_RET_DATA)),
        BPF_JUMP(BPF_JMP | BPF_JEQ | BPF_K, SYS_clone3, 0, 1),
        BPF_STMT(BPF_RET | BPF_K,
                 SECCOMP_RET_ERRNO | (ENOSYS & SECCOMP_RET_DATA)),
        ALLOW_SYSCALL(read),
        ALLOW_SYSCALL(write),
        ALLOW_SYSCALL(close),
        ALLOW_SYSCALL(fstat),
        ALLOW_SYSCALL(newfstatat),
        ALLOW_SYSCALL(lseek),
        ALLOW_SYSCALL(mmap),
        ALLOW_SYSCALL(mprotect),
        ALLOW_SYSCALL(munmap),
        ALLOW_SYSCALL(brk),
        ALLOW_SYSCALL(rt_sigaction),
        ALLOW_SYSCALL(rt_sigprocmask),
        ALLOW_SYSCALL(rt_sigreturn),
        ALLOW_SYSCALL(sigaltstack),
        ALLOW_SYSCALL(ioctl),
        ALLOW_SYSCALL(pread64),
        ALLOW_SYSCALL(access),
        ALLOW_SYSCALL(openat),
        ALLOW_SYSCALL(readlink),
        ALLOW_SYSCALL(readlinkat),
        ALLOW_SYSCALL(getcwd),
        ALLOW_SYSCALL(fcntl),
        ALLOW_SYSCALL(dup),
        ALLOW_SYSCALL(dup2),
        ALLOW_SYSCALL(dup3),
        ALLOW_SYSCALL(pipe),
        ALLOW_SYSCALL(pipe2),
        ALLOW_SYSCALL(poll),
        ALLOW_SYSCALL(ppoll),
        ALLOW_SYSCALL(select),
        ALLOW_SYSCALL(pselect6),
        ALLOW_SYSCALL(sched_yield),
        ALLOW_SYSCALL(mremap),
        ALLOW_SYSCALL(madvise),
        ALLOW_SYSCALL(mincore),
        ALLOW_SYSCALL(prctl),
        ALLOW_SYSCALL(arch_prctl),
        ALLOW_SYSCALL(set_tid_address),
        ALLOW_SYSCALL(set_robust_list),
        ALLOW_SYSCALL(rseq),
        ALLOW_SYSCALL(prlimit64),
        ALLOW_SYSCALL(getrandom),
        ALLOW_SYSCALL(futex),
        ALLOW_SYSCALL(clock_gettime),
        ALLOW_SYSCALL(clock_nanosleep),
        ALLOW_SYSCALL(nanosleep),
        ALLOW_SYSCALL(sched_getaffinity),
        ALLOW_SYSCALL(getpid),
        ALLOW_SYSCALL(gettid),
        ALLOW_SYSCALL(tgkill),
        ALLOW_SYSCALL(uname),
        ALLOW_SYSCALL(sysinfo),
        ALLOW_SYSCALL(getdents64),
        ALLOW_SYSCALL(statx),
        ALLOW_SYSCALL(membarrier),
        ALLOW_SYSCALL(execve),
        ALLOW_SYSCALL(exit),
        ALLOW_SYSCALL(exit_group),
        BPF_STMT(BPF_RET | BPF_K,
                 SECCOMP_RET_ERRNO | (EPERM & SECCOMP_RET_DATA)),
    };
    const struct sock_fprog program = {
        .len = (unsigned short)(sizeof(filter) / sizeof(filter[0])),
        .filter = filter,
    };
    return syscall(SYS_seccomp, SECCOMP_SET_MODE_FILTER, 0, &program) == 0;
}

static bool set_limits(void) {
    const struct rlimit address_space = {.rlim_cur = 536870912,
                                         .rlim_max = 536870912};
    const struct rlimit file_size = {.rlim_cur = 131072, .rlim_max = 131072};
    const struct rlimit descriptors = {.rlim_cur = 64, .rlim_max = 64};
    const struct rlimit cpu = {.rlim_cur = 8, .rlim_max = 8};
    return setrlimit(RLIMIT_AS, &address_space) == 0 &&
           setrlimit(RLIMIT_FSIZE, &file_size) == 0 &&
           setrlimit(RLIMIT_NOFILE, &descriptors) == 0 &&
           setrlimit(RLIMIT_CPU, &cpu) == 0;
}

static bool path_is_regular(const char *path) {
    struct stat status;
    return lstat(path, &status) == 0 && S_ISREG(status.st_mode) &&
           status.st_nlink == 1;
}

static bool denied_read(const char *path) {
    errno = 0;
    const int fd = open(path, O_RDONLY | O_CLOEXEC);
    const int observed = errno;
    if (fd >= 0) {
        (void)close(fd);
        return false;
    }
    return observed == EACCES || observed == EPERM;
}

static void run_child(char *const arguments[]) {
    const char *job_root = arguments[2];
    const char *yr_path = arguments[3];
    const char *rules_path = arguments[4];
    const char *artifact_path = arguments[5];
    const char *external_path = arguments[6];
    const char *credential_path = arguments[7];
    const char *write_probe = arguments[8];

    if (!path_is_regular(yr_path) || !path_is_regular(rules_path) ||
        !path_is_regular(artifact_path) ||
        prctl(PR_SET_NO_NEW_PRIVS, 1, 0, 0, 0) != 0 || !set_limits() ||
        !install_read_execute_landlock(job_root)) {
        _exit(40);
    }

    close_unrelated_descriptors();
    const bool external_denied = denied_read(external_path);
    const bool credential_denied = denied_read(credential_path);
    errno = 0;
    const int write_fd = open(write_probe,
                              O_WRONLY | O_CREAT | O_EXCL | O_CLOEXEC, 0600);
    const bool write_denied = write_fd < 0 &&
                              (errno == EACCES || errno == EPERM);
    if (write_fd >= 0) {
        (void)close(write_fd);
    }
    if (!external_denied || !credential_denied || !write_denied ||
        !install_seccomp()) {
        _exit(41);
    }

    errno = 0;
    const int socket_fd = socket(AF_INET, SOCK_STREAM, 0);
    const bool network_denied = socket_fd < 0 && errno == EPERM;
    if (socket_fd >= 0) {
        (void)close(socket_fd);
    }
    if (!network_denied) {
        _exit(42);
    }

    static const char marker[] =
        "atomic_cgroup_placement=true\n"
        "landlock_read_only_job=true\n"
        "external_filesystem_denied=true\n"
        "credential_denied=true\n"
        "writable_filesystem_denied=true\n"
        "network_denied=true\n"
        "unrelated_descriptors_closed=true\n";
    if (write(STDERR_FILENO, marker, sizeof(marker) - 1) !=
        (ssize_t)(sizeof(marker) - 1)) {
        _exit(43);
    }

    char home[4096];
    const int written = snprintf(home, sizeof(home), "HOME=%s/home", job_root);
    if (written <= 0 || (size_t)written >= sizeof(home)) {
        _exit(44);
    }
    char *const environment[] = {home, "LANG=C", "LC_ALL=C", NULL};
    char *const scan_arguments[] = {
        (char *)yr_path,
        "scan",
        "--compiled-rules",
        "--output-format=ndjson",
        "--print-namespace",
        "--print-tags",
        "--print-strings=0",
        "--disable-console-logs",
        "--no-mmap",
        "--max-matches-per-pattern=32",
        "--threads=1",
        "--timeout=5",
        "--skip-larger=262144",
        (char *)rules_path,
        (char *)artifact_path,
        NULL,
    };
    execve(yr_path, scan_arguments, environment);
    _exit(45);
}

int main(int argument_count, char *const arguments[]) {
    if (argument_count != 9) {
        return 2;
    }
    const int cgroup_fd = open(arguments[1], O_RDONLY | O_DIRECTORY | O_CLOEXEC);
    if (cgroup_fd < 0) {
        return 30;
    }
    const int inherited_fd = open(arguments[6], O_RDONLY | O_CLOEXEC);
    if (inherited_fd < 3) {
        (void)close(cgroup_fd);
        return 31;
    }
    const struct clone_args clone_arguments = {
        .flags = CLONE_INTO_CGROUP,
        .exit_signal = SIGCHLD,
        .cgroup = (uint64_t)cgroup_fd,
    };
    const pid_t worker =
        (pid_t)syscall(SYS_clone3, &clone_arguments, sizeof(clone_arguments));
    if (worker < 0) {
        (void)close(inherited_fd);
        (void)close(cgroup_fd);
        return 32;
    }
    if (worker == 0) {
        run_child(arguments);
    }
    (void)close(inherited_fd);
    (void)close(cgroup_fd);
    int status = 0;
    while (waitpid(worker, &status, 0) < 0) {
        if (errno != EINTR) {
            return 33;
        }
    }
    if (!WIFEXITED(status)) {
        return 34;
    }
    return WEXITSTATUS(status);
}
