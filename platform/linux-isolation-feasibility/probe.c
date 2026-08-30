// SPDX-License-Identifier: Apache-2.0
#define _GNU_SOURCE

#include <errno.h>
#include <fcntl.h>
#include <linux/audit.h>
#include <linux/filter.h>
#include <linux/landlock.h>
#include <linux/magic.h>
#include <linux/sched.h>
#include <linux/seccomp.h>
#include <signal.h>
#include <stddef.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/prctl.h>
#include <sys/socket.h>
#include <sys/stat.h>
#include <sys/statfs.h>
#include <sys/syscall.h>
#include <sys/utsname.h>
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
#ifndef CLONE_INTO_CGROUP
#define CLONE_INTO_CGROUP 0x200000000ULL
#endif
#ifndef SYS_clone3
#define SYS_clone3 435
#endif

#if defined(__x86_64__)
#define PROBE_ARCHITECTURE "x86_64"
#define PROBE_AUDIT_ARCH AUDIT_ARCH_X86_64
#define PROBE_ARCHITECTURE_FILTER true
#elif defined(__aarch64__)
#define PROBE_ARCHITECTURE "aarch64"
#define PROBE_AUDIT_ARCH AUDIT_ARCH_AARCH64
#define PROBE_ARCHITECTURE_FILTER true
#else
#define PROBE_ARCHITECTURE "unsupported"
#define PROBE_AUDIT_ARCH 0
#define PROBE_ARCHITECTURE_FILTER false
#endif

#define BOOLEAN(value) ((value) ? "true" : "false")

static int landlock_create_ruleset(const struct landlock_ruleset_attr *const attr,
                                   const size_t size,
                                   const uint32_t flags) {
    return (int)syscall(SYS_landlock_create_ruleset, attr, size, flags);
}

static int landlock_add_rule(const int ruleset_fd,
                             const enum landlock_rule_type rule_type,
                             const void *const rule_attr,
                             const uint32_t flags) {
    return (int)syscall(SYS_landlock_add_rule, ruleset_fd, rule_type, rule_attr,
                        flags);
}

static int landlock_restrict_self(const int ruleset_fd,
                                  const uint32_t flags) {
    return (int)syscall(SYS_landlock_restrict_self, ruleset_fd, flags);
}

static int landlock_abi(void) {
    const int abi = landlock_create_ruleset(NULL, 0,
                                            LANDLOCK_CREATE_RULESET_VERSION);
    return abi < 0 ? 0 : abi;
}

static bool token_present(const char *const values, const char *const token) {
    const size_t token_length = strlen(token);
    const char *cursor = values;
    while (*cursor != '\0') {
        while (*cursor == ' ' || *cursor == '\n' || *cursor == '\t') {
            ++cursor;
        }
        if (strncmp(cursor, token, token_length) == 0 &&
            (cursor[token_length] == '\0' || cursor[token_length] == ' ' ||
             cursor[token_length] == '\n' || cursor[token_length] == '\t')) {
            return true;
        }
        while (*cursor != '\0' && *cursor != ' ' && *cursor != '\n' &&
               *cursor != '\t') {
            ++cursor;
        }
    }
    return false;
}

static bool read_small_file(const char *const path, char *const buffer,
                            const size_t capacity) {
    const int descriptor = open(path, O_RDONLY | O_CLOEXEC);
    if (descriptor < 0) {
        return false;
    }
    const ssize_t length = read(descriptor, buffer, capacity - 1);
    const int saved_errno = errno;
    (void)close(descriptor);
    errno = saved_errno;
    if (length < 0) {
        return false;
    }
    buffer[(size_t)length] = '\0';
    return true;
}

static bool safe_cgroup_suffix(const char *const suffix) {
    if (suffix[0] != '/' || strstr(suffix, "..") != NULL) {
        return false;
    }
    for (const unsigned char *cursor = (const unsigned char *)suffix;
         *cursor != '\0'; ++cursor) {
        const unsigned char value = *cursor;
        if ((value >= 'a' && value <= 'z') ||
            (value >= 'A' && value <= 'Z') ||
            (value >= '0' && value <= '9') || value == '/' || value == '_' ||
            value == '-' || value == '.' || value == '\\') {
            continue;
        }
        return false;
    }
    return true;
}

static bool current_cgroup_directory(char *const destination,
                                     const size_t capacity) {
    char membership[4096];
    if (!read_small_file("/proc/self/cgroup", membership, sizeof(membership))) {
        return false;
    }
    char *line = membership;
    while (line != NULL && *line != '\0') {
        char *next = strchr(line, '\n');
        if (next != NULL) {
            *next = '\0';
            ++next;
        }
        if (strncmp(line, "0::", 3) == 0 && safe_cgroup_suffix(line + 3)) {
            const int written = snprintf(destination, capacity,
                                         "/sys/fs/cgroup%s", line + 3);
            return written > 0 && (size_t)written < capacity;
        }
        line = next;
    }
    return false;
}

static bool joined_path(char *const destination, const size_t capacity,
                        const char *const directory, const char *const name) {
    const int written = snprintf(destination, capacity, "%s/%s", directory,
                                 name);
    return written > 0 && (size_t)written < capacity;
}

static void print_capabilities(void) {
    struct utsname observed;
    if (uname(&observed) != 0) {
        exit(10);
    }

    const int abi = landlock_abi();
    const bool no_new_privs = prctl(PR_GET_NO_NEW_PRIVS, 0, 0, 0, 0) >= 0;
    uint32_t action = SECCOMP_RET_KILL_PROCESS;
    const bool seccomp_filter =
        syscall(SYS_seccomp, SECCOMP_GET_ACTION_AVAIL, 0, &action) == 0;

    struct statfs cgroup_stat;
    const bool cgroup_v2 =
        statfs("/sys/fs/cgroup", &cgroup_stat) == 0 &&
        (unsigned long)cgroup_stat.f_type == (unsigned long)CGROUP2_SUPER_MAGIC;
    char cgroup_directory[4096] = {0};
    const bool cgroup_path =
        cgroup_v2 &&
        current_cgroup_directory(cgroup_directory, sizeof(cgroup_directory));
    char controller_path[4096] = {0};
    char controllers[4096] = {0};
    const bool controller_path_valid =
        cgroup_path && joined_path(controller_path, sizeof(controller_path),
                                   cgroup_directory, "cgroup.controllers");
    const bool controller_inventory =
        controller_path_valid &&
        read_small_file(controller_path, controllers, sizeof(controllers));
    const bool cpu = controller_inventory && token_present(controllers, "cpu");
    const bool memory =
        controller_inventory && token_present(controllers, "memory");
    const bool pids =
        controller_inventory && token_present(controllers, "pids");

    char subtree_path[4096] = {0};
    char procs_path[4096] = {0};
    char kill_path[4096] = {0};
    char events_path[4096] = {0};
    const bool control_paths =
        cgroup_path &&
        joined_path(subtree_path, sizeof(subtree_path), cgroup_directory,
                    "cgroup.subtree_control") &&
        joined_path(procs_path, sizeof(procs_path), cgroup_directory,
                    "cgroup.procs") &&
        joined_path(kill_path, sizeof(kill_path), cgroup_directory,
                    "cgroup.kill") &&
        joined_path(events_path, sizeof(events_path), cgroup_directory,
                    "cgroup.events");
    const bool delegated = control_paths && access(cgroup_directory, W_OK) == 0 &&
                           access(subtree_path, W_OK) == 0 &&
                           access(procs_path, W_OK) == 0;
    const bool cgroup_kill = control_paths && access(kill_path, F_OK) == 0;
    const bool cgroup_events = control_paths && access(events_path, R_OK) == 0;

    printf("kernel_release=%s\n", observed.release);
    printf("architecture=%s\n", PROBE_ARCHITECTURE);
    printf("landlock_abi=%d\n", abi);
    printf("no_new_privs=%s\n", BOOLEAN(no_new_privs));
    printf("landlock=%s\n", BOOLEAN(abi >= 1));
    printf("seccomp_filter=%s\n", BOOLEAN(seccomp_filter));
    printf("seccomp_kill_process=%s\n", BOOLEAN(seccomp_filter));
    printf("architecture_filter=%s\n", BOOLEAN(PROBE_ARCHITECTURE_FILTER));
    printf("cgroup_v2=%s\n", BOOLEAN(cgroup_v2));
    printf("cpu_controller=%s\n", BOOLEAN(cpu));
    printf("memory_controller=%s\n", BOOLEAN(memory));
    printf("pids_controller=%s\n", BOOLEAN(pids));
    printf("delegated_leaf=%s\n", BOOLEAN(delegated));
    printf("cgroup_kill=%s\n", BOOLEAN(cgroup_kill));
    printf("cgroup_empty_verification=%s\n", BOOLEAN(cgroup_events));
}

static uint64_t handled_filesystem_access(const int abi) {
    (void)abi;
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

static bool install_read_only_landlock(const char *const allowed_directory,
                                       const int abi) {
    const struct landlock_ruleset_attr ruleset = {
        .handled_access_fs = handled_filesystem_access(abi),
    };
    const int ruleset_fd =
        landlock_create_ruleset(&ruleset, sizeof(ruleset), 0);
    if (ruleset_fd < 0) {
        return false;
    }
    const int directory_fd =
        open(allowed_directory, O_PATH | O_CLOEXEC | O_DIRECTORY);
    if (directory_fd < 0) {
        (void)close(ruleset_fd);
        return false;
    }
    const struct landlock_path_beneath_attr path_rule = {
        .allowed_access = LANDLOCK_ACCESS_FS_READ_FILE |
                          LANDLOCK_ACCESS_FS_READ_DIR,
        .parent_fd = directory_fd,
    };
    const bool added =
        landlock_add_rule(ruleset_fd, LANDLOCK_RULE_PATH_BENEATH, &path_rule,
                          0) == 0;
    const bool restricted =
        added && landlock_restrict_self(ruleset_fd, 0) == 0;
    (void)close(directory_fd);
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
    for (int descriptor = 3; descriptor < upper; ++descriptor) {
        (void)close(descriptor);
    }
}

#if defined(__x86_64__) || defined(__aarch64__)
#define ALLOW_SYSCALL(name)                                                   \
    BPF_JUMP(BPF_JMP | BPF_JEQ | BPF_K, SYS_##name, 0, 1),                  \
        BPF_STMT(BPF_RET | BPF_K, SECCOMP_RET_ALLOW)

static bool install_synthetic_seccomp(void) {
    struct sock_filter filter[] = {
        BPF_STMT(BPF_LD | BPF_W | BPF_ABS,
                 (uint32_t)offsetof(struct seccomp_data, arch)),
        BPF_JUMP(BPF_JMP | BPF_JEQ | BPF_K, PROBE_AUDIT_ARCH, 1, 0),
        BPF_STMT(BPF_RET | BPF_K, SECCOMP_RET_KILL_PROCESS),
        BPF_STMT(BPF_LD | BPF_W | BPF_ABS,
                 (uint32_t)offsetof(struct seccomp_data, nr)),
        ALLOW_SYSCALL(read),
        ALLOW_SYSCALL(write),
        ALLOW_SYSCALL(close),
        ALLOW_SYSCALL(openat),
        ALLOW_SYSCALL(newfstatat),
        ALLOW_SYSCALL(fcntl),
        ALLOW_SYSCALL(rt_sigprocmask),
        ALLOW_SYSCALL(rt_sigreturn),
        ALLOW_SYSCALL(exit),
        ALLOW_SYSCALL(exit_group),
        BPF_STMT(BPF_RET | BPF_K, SECCOMP_RET_ERRNO | (EPERM & SECCOMP_RET_DATA)),
    };
    const struct sock_fprog program = {
        .len = (unsigned short)(sizeof(filter) / sizeof(filter[0])),
        .filter = filter,
    };
    return syscall(SYS_seccomp, SECCOMP_SET_MODE_FILTER, 0, &program) == 0;
}
#else
static bool install_synthetic_seccomp(void) { return false; }
#endif

static bool open_read_succeeds(const char *const path) {
    const int descriptor = open(path, O_RDONLY | O_CLOEXEC);
    if (descriptor < 0) {
        return false;
    }
    char byte = '\0';
    const bool read_succeeded = read(descriptor, &byte, 1) == 1;
    (void)close(descriptor);
    return read_succeeded;
}

static bool open_read_is_denied(const char *const path) {
    errno = 0;
    const int descriptor = open(path, O_RDONLY | O_CLOEXEC);
    const int observed_errno = errno;
    if (descriptor >= 0) {
        (void)close(descriptor);
        return false;
    }
    return observed_errno == EACCES || observed_errno == EPERM;
}

static void run_primitive_probe(const int argument_count,
                                char *const arguments[]) {
    if (argument_count != 7) {
        exit(20);
    }
    const char *const allowed_directory = arguments[2];
    const char *const allowed_file = arguments[3];
    const char *const external_file = arguments[4];
    const char *const credential_file = arguments[5];
    const char *const write_probe = arguments[6];

    const int inherited_descriptor =
        open(external_file, O_RDONLY | O_CLOEXEC);
    if (inherited_descriptor < 3) {
        exit(21);
    }
    const int abi = landlock_abi();
    if (abi < 1 || prctl(PR_SET_NO_NEW_PRIVS, 1, 0, 0, 0) != 0 ||
        !install_read_only_landlock(allowed_directory, abi)) {
        exit(22);
    }
    close_unrelated_descriptors();
    errno = 0;
    const bool descriptors_closed =
        fcntl(inherited_descriptor, F_GETFD) == -1 && errno == EBADF;
    const bool no_new_privs_effective =
        prctl(PR_GET_NO_NEW_PRIVS, 0, 0, 0, 0) == 1;
    if (!install_synthetic_seccomp()) {
        exit(23);
    }

    const bool allowed_read = open_read_succeeds(allowed_file);
    const bool external_denied = open_read_is_denied(external_file);
    const bool credential_denied = open_read_is_denied(credential_file);
    const bool device_denied = open_read_is_denied("/dev/null");

    errno = 0;
    const int socket_descriptor = socket(AF_INET, SOCK_STREAM, 0);
    const bool network_denied = socket_descriptor == -1 && errno == EPERM;
    if (socket_descriptor >= 0) {
        (void)close(socket_descriptor);
    }
    errno = 0;
    const pid_t child = fork();
    const bool descendant_denied = child == -1 && errno == EPERM;
    if (child == 0) {
        _exit(99);
    }

    errno = 0;
    const int write_descriptor =
        open(write_probe, O_WRONLY | O_CREAT | O_EXCL | O_CLOEXEC, 0600);
    const bool zero_writable_filesystem =
        write_descriptor == -1 && (errno == EACCES || errno == EPERM);
    if (write_descriptor >= 0) {
        (void)close(write_descriptor);
    }

    char output[1024];
    const int output_length = snprintf(
        output, sizeof(output),
        "no_new_privs_effective=%s\n"
        "landlock_read_only_input=%s\n"
        "external_filesystem_denial=%s\n"
        "credential_denial=%s\n"
        "device_denial=%s\n"
        "network_denial=%s\n"
        "unrelated_descriptors_closed=%s\n"
        "descendant_denial=%s\n"
        "zero_writable_filesystem=%s\n",
        BOOLEAN(no_new_privs_effective), BOOLEAN(allowed_read),
        BOOLEAN(external_denied), BOOLEAN(credential_denied),
        BOOLEAN(device_denied), BOOLEAN(network_denied),
        BOOLEAN(descriptors_closed), BOOLEAN(descendant_denied),
        BOOLEAN(zero_writable_filesystem));
    if (output_length <= 0 || (size_t)output_length >= sizeof(output) ||
        write(STDOUT_FILENO, output, (size_t)output_length) != output_length) {
        _exit(24);
    }
    const bool passed = no_new_privs_effective && allowed_read &&
                        external_denied && credential_denied && device_denied &&
                        network_denied && descriptors_closed &&
                        descendant_denied && zero_writable_filesystem;
    _exit(passed ? 0 : 25);
}

static int run_composite_probe(const int argument_count,
                               char *const arguments[]) {
    if (argument_count != 8) {
        return 30;
    }
    const int cgroup_fd =
        open(arguments[2], O_RDONLY | O_DIRECTORY | O_CLOEXEC);
    if (cgroup_fd < 0) {
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
        (void)close(cgroup_fd);
        return 32;
    }
    if (worker == 0) {
        (void)close(cgroup_fd);
        char *primitive_arguments[] = {
            arguments[0], "primitive", arguments[3], arguments[4],
            arguments[5], arguments[6], arguments[7],
        };
        run_primitive_probe(7, primitive_arguments);
    }
    (void)close(cgroup_fd);
    int status = 0;
    while (waitpid(worker, &status, 0) < 0) {
        if (errno != EINTR) {
            return 33;
        }
    }
    return WIFEXITED(status) ? WEXITSTATUS(status) : 34;
}

int main(const int argument_count, char *const arguments[]) {
    if (argument_count == 2 && strcmp(arguments[1], "capabilities") == 0) {
        print_capabilities();
        return 0;
    }
    if (argument_count >= 2 && strcmp(arguments[1], "primitive") == 0) {
        run_primitive_probe(argument_count, arguments);
        return 0;
    }
    if (argument_count >= 2 && strcmp(arguments[1], "composite") == 0) {
        return run_composite_probe(argument_count, arguments);
    }
    return 2;
}
