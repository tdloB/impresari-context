// SPDX-License-Identifier: Apache-2.0
#define _GNU_SOURCE

#include <dirent.h>
#include <errno.h>
#include <fcntl.h>
#include <linux/reboot.h>
#include <signal.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/mman.h>
#include <sys/mount.h>
#include <sys/reboot.h>
#include <sys/stat.h>
#include <sys/syscall.h>
#include <sys/types.h>
#include <sys/wait.h>
#include <time.h>
#include <unistd.h>

#define INPUT_BYTES 4096
#define SCRATCH_BYTES 1048576
#define MEMORY_LIMIT_BYTES 33554432
#define MEMORY_PRESSURE_BYTES 134217728
#define CPU_QUOTA_MICROSECONDS 10000
#define CPU_PERIOD_MICROSECONDS 100000
#define PIDS_LIMIT 8

static const char *const canary_markers[] = {
    "IMPRESARI_HOST_HOME_CANARY_V1",
    "IMPRESARI_HOST_REPOSITORY_CANARY_V1",
    "IMPRESARI_HOST_CACHE_CANARY_V1",
    "IMPRESARI_HOST_CREDENTIAL_CANARY_V1",
    "IMPRESARI_HOST_DEVICE_CANARY_V1",
    "IMPRESARI_HOST_PROCESS_CANARY_V1",
};

static bool write_text(const char *path, const char *value) {
    int fd = open(path, O_WRONLY | O_CLOEXEC);
    if (fd < 0) {
        return false;
    }
    size_t length = strlen(value);
    bool passed = write(fd, value, length) == (ssize_t)length;
    close(fd);
    return passed;
}

static bool load_block_driver(void) {
    int fd = open("/lib/modules/virtio_blk.ko", O_RDONLY | O_CLOEXEC);
    if (fd < 0) {
        return false;
    }
    int result = (int)syscall(SYS_finit_module, fd, "", 0);
    int saved_errno = errno;
    close(fd);
    return result == 0 || saved_errno == EEXIST;
}

static bool wait_for_path(const char *path) {
    for (int attempt = 0; attempt < 500; attempt++) {
        if (access(path, F_OK) == 0) {
            return true;
        }
        usleep(10000);
    }
    return false;
}

static bool exact_block_devices(void) {
    DIR *directory = opendir("/sys/block");
    if (directory == NULL) {
        return false;
    }
    bool vda = false;
    bool vdb = false;
    bool unexpected = false;
    struct dirent *entry;
    while ((entry = readdir(directory)) != NULL) {
        if (strcmp(entry->d_name, ".") == 0 || strcmp(entry->d_name, "..") == 0) {
            continue;
        }
        if (strcmp(entry->d_name, "vda") == 0) {
            vda = true;
        } else if (strcmp(entry->d_name, "vdb") == 0) {
            vdb = true;
        } else {
            unexpected = true;
        }
    }
    closedir(directory);
    return vda && vdb && !unexpected;
}

static bool block_has_no_canary(const char *path, size_t bytes) {
    unsigned char *buffer = malloc(bytes);
    if (buffer == NULL) {
        return false;
    }
    int fd = open(path, O_RDONLY | O_CLOEXEC);
    if (fd < 0) {
        free(buffer);
        return false;
    }
    ssize_t amount = pread(fd, buffer, bytes, 0);
    close(fd);
    if (amount != (ssize_t)bytes) {
        free(buffer);
        return false;
    }
    bool absent = true;
    for (size_t index = 0; index < sizeof(canary_markers) / sizeof(canary_markers[0]); index++) {
        size_t marker_bytes = strlen(canary_markers[index]);
        if (memmem(buffer, bytes, canary_markers[index], marker_bytes) != NULL) {
            absent = false;
        }
    }
    free(buffer);
    return absent;
}

static bool host_paths_absent(void) {
    static const char *const paths[] = {
        "/Users", "/private", "/Volumes", "/System", "/Applications",
        "/host-canary", "/run/host", "/mnt/host", "/root/.ssh",
        "/root/.config/gh", "/root/.aws", "/root/.claude",
        "/var/run/docker.sock",
    };
    for (size_t index = 0; index < sizeof(paths) / sizeof(paths[0]); index++) {
        if (access(paths[index], F_OK) == 0 || errno != ENOENT) {
            return false;
        }
    }
    return true;
}

static bool host_process_invisible(void) {
    DIR *directory = opendir("/proc");
    if (directory == NULL) {
        return false;
    }
    bool absent = true;
    struct dirent *entry;
    while ((entry = readdir(directory)) != NULL) {
        char *end = NULL;
        (void)strtol(entry->d_name, &end, 10);
        if (end == entry->d_name || *end != '\0') {
            continue;
        }
        char path[128];
        int amount = snprintf(path, sizeof(path), "/proc/%s/cmdline", entry->d_name);
        if (amount <= 0 || (size_t)amount >= sizeof(path)) {
            absent = false;
            break;
        }
        int fd = open(path, O_RDONLY | O_CLOEXEC);
        if (fd < 0) {
            continue;
        }
        char command_line[1024];
        ssize_t bytes = read(fd, command_line, sizeof(command_line) - 1);
        close(fd);
        if (bytes > 0) {
            command_line[bytes] = '\0';
            if (memmem(command_line, (size_t)bytes,
                       "impresari-context-vm-controller",
                       strlen("impresari-context-vm-controller")) != NULL) {
                absent = false;
                break;
            }
        }
    }
    closedir(directory);
    return absent;
}

static bool read_stat_value(const char *path, const char *key, uint64_t *value) {
    FILE *file = fopen(path, "re");
    if (file == NULL) {
        return false;
    }
    char name[64];
    unsigned long long parsed;
    bool found = false;
    while (fscanf(file, "%63s %llu", name, &parsed) == 2) {
        if (strcmp(name, key) == 0) {
            *value = (uint64_t)parsed;
            found = true;
            break;
        }
    }
    fclose(file);
    return found;
}

static bool read_single_value(const char *path, uint64_t *value) {
    FILE *file = fopen(path, "re");
    if (file == NULL) {
        return false;
    }
    unsigned long long parsed;
    bool passed = fscanf(file, "%llu", &parsed) == 1;
    fclose(file);
    if (passed) {
        *value = (uint64_t)parsed;
    }
    return passed;
}

static bool place_process(pid_t process) {
    char value[32];
    int amount = snprintf(value, sizeof(value), "%ld", (long)process);
    return amount > 0 && (size_t)amount < sizeof(value) &&
           write_text("/sys/fs/cgroup/impresari-job/cgroup.procs", value);
}

static bool wait_bounded(pid_t child, int *status, int milliseconds) {
    for (int elapsed = 0; elapsed < milliseconds; elapsed += 10) {
        pid_t result = waitpid(child, status, WNOHANG);
        if (result == child) {
            return true;
        }
        if (result < 0) {
            return false;
        }
        usleep(10000);
    }
    kill(child, SIGKILL);
    return waitpid(child, status, 0) == child;
}

static bool memory_pressure_contained(uint64_t *oom_kills) {
    int gate[2];
    if (pipe2(gate, O_CLOEXEC) != 0) {
        return false;
    }
    pid_t child = fork();
    if (child < 0) {
        close(gate[0]);
        close(gate[1]);
        return false;
    }
    if (child == 0) {
        close(gate[1]);
        char token;
        if (read(gate[0], &token, 1) != 1) {
            _exit(41);
        }
        close(gate[0]);
        unsigned char *memory = mmap(NULL, MEMORY_PRESSURE_BYTES, PROT_READ | PROT_WRITE,
                                     MAP_PRIVATE | MAP_ANONYMOUS, -1, 0);
        if (memory == MAP_FAILED) {
            _exit(42);
        }
        for (size_t offset = 0; offset < MEMORY_PRESSURE_BYTES; offset += 4096) {
            memory[offset] = (unsigned char)(offset >> 12);
        }
        _exit(43);
    }
    close(gate[0]);
    bool placed = place_process(child);
    char token = 'm';
    bool released = write(gate[1], &token, 1) == 1;
    close(gate[1]);
    int status = 0;
    bool reaped = wait_bounded(child, &status, 5000);
    uint64_t kills = 0;
    bool observed = read_stat_value("/sys/fs/cgroup/impresari-job/memory.events",
                                    "oom_kill", &kills);
    *oom_kills = kills;
    return placed && released && reaped && observed && kills >= 1 &&
           WIFSIGNALED(status) && WTERMSIG(status) == SIGKILL;
}

static uint64_t monotonic_nanoseconds(void) {
    struct timespec value;
    if (clock_gettime(CLOCK_MONOTONIC, &value) != 0) {
        return 0;
    }
    return (uint64_t)value.tv_sec * 1000000000ULL + (uint64_t)value.tv_nsec;
}

static bool cpu_pressure_bounded(uint64_t *usage_delta, uint64_t *throttled_delta) {
    uint64_t usage_before = 0;
    uint64_t throttled_before = 0;
    if (!read_stat_value("/sys/fs/cgroup/impresari-job/cpu.stat", "usage_usec", &usage_before) ||
        !read_stat_value("/sys/fs/cgroup/impresari-job/cpu.stat", "nr_throttled", &throttled_before)) {
        return false;
    }
    int gate[2];
    if (pipe2(gate, O_CLOEXEC) != 0) {
        return false;
    }
    pid_t child = fork();
    if (child < 0) {
        close(gate[0]);
        close(gate[1]);
        return false;
    }
    if (child == 0) {
        close(gate[1]);
        char token;
        if (read(gate[0], &token, 1) != 1) {
            _exit(51);
        }
        close(gate[0]);
        uint64_t start = monotonic_nanoseconds();
        volatile uint64_t accumulator = 1;
        while (monotonic_nanoseconds() - start < 1500000000ULL) {
            accumulator = accumulator * 6364136223846793005ULL + 1;
        }
        _exit(accumulator == 0 ? 52 : 0);
    }
    close(gate[0]);
    bool placed = place_process(child);
    char token = 'c';
    bool released = write(gate[1], &token, 1) == 1;
    close(gate[1]);
    int status = 0;
    bool reaped = wait_bounded(child, &status, 4000);
    uint64_t usage_after = 0;
    uint64_t throttled_after = 0;
    bool measured = read_stat_value("/sys/fs/cgroup/impresari-job/cpu.stat", "usage_usec", &usage_after) &&
                    read_stat_value("/sys/fs/cgroup/impresari-job/cpu.stat", "nr_throttled", &throttled_after);
    if (!measured || usage_after < usage_before || throttled_after < throttled_before) {
        return false;
    }
    *usage_delta = usage_after - usage_before;
    *throttled_delta = throttled_after - throttled_before;
    return placed && released && reaped && WIFEXITED(status) && WEXITSTATUS(status) == 0 &&
           *usage_delta >= 50000 && *usage_delta <= 400000 && *throttled_delta >= 1;
}

static bool configure_job_cgroup(void) {
    if (mkdir("/sys/fs/cgroup/impresari-job", 0755) != 0) {
        return false;
    }
    return write_text("/sys/fs/cgroup/impresari-job/memory.max", "33554432") &&
           write_text("/sys/fs/cgroup/impresari-job/memory.swap.max", "0") &&
           write_text("/sys/fs/cgroup/impresari-job/pids.max", "8") &&
           write_text("/sys/fs/cgroup/impresari-job/cpu.max", "10000 100000");
}

static void power_off(void) {
    sync();
    reboot(LINUX_REBOOT_CMD_POWER_OFF);
    for (;;) {
        pause();
    }
}

int main(void) {
    mkdir("/dev", 0755);
    mkdir("/proc", 0555);
    mkdir("/sys", 0555);
    bool dev_ready = mount("devtmpfs", "/dev", "devtmpfs", MS_NOSUID | MS_NOEXEC,
                           "mode=0755") == 0 || errno == EBUSY;
    bool proc_ready = mount("proc", "/proc", "proc", MS_NOSUID | MS_NODEV | MS_NOEXEC, "") == 0 ||
                      errno == EBUSY;
    bool sys_ready = mount("sysfs", "/sys", "sysfs", MS_NOSUID | MS_NODEV | MS_NOEXEC, "") == 0 ||
                     errno == EBUSY;
    mkdir("/sys/fs/cgroup", 0755);
    bool cgroup_ready = mount("cgroup2", "/sys/fs/cgroup", "cgroup2",
                              MS_NOSUID | MS_NODEV | MS_NOEXEC, "") == 0 || errno == EBUSY;
    bool controllers = cgroup_ready &&
        write_text("/sys/fs/cgroup/cgroup.subtree_control", "+cpu +memory +pids");
    bool driver_loaded = dev_ready && load_block_driver();
    bool disks_ready = driver_loaded && wait_for_path("/dev/vda") && wait_for_path("/dev/vdb");
    bool exact_devices = sys_ready && disks_ready && exact_block_devices();
    bool no_canary_bytes = disks_ready && block_has_no_canary("/dev/vda", INPUT_BYTES) &&
                           block_has_no_canary("/dev/vdb", SCRATCH_BYTES);
    bool paths_absent = host_paths_absent();
    bool process_absent = proc_ready && host_process_invisible();
    bool configured = controllers && configure_job_cgroup();
    uint64_t oom_kills = 0;
    uint64_t cpu_usage = 0;
    uint64_t cpu_throttled = 0;
    bool memory_contained = configured && memory_pressure_contained(&oom_kills);
    bool cpu_bounded = configured && cpu_pressure_bounded(&cpu_usage, &cpu_throttled);
    uint64_t pids_peak = 0;
    bool pids_bounded = configured &&
        read_single_value("/sys/fs/cgroup/impresari-job/pids.peak", &pids_peak) &&
        pids_peak <= PIDS_LIMIT;
    bool cgroup_empty = configured && write_text("/sys/fs/cgroup/impresari-job/cgroup.kill", "1");
    bool cgroup_removed = cgroup_empty && rmdir("/sys/fs/cgroup/impresari-job") == 0;

    bool passed = exact_devices && no_canary_bytes && paths_absent && process_absent &&
                  memory_contained && cpu_bounded && pids_bounded && cgroup_removed;
    printf("IMPRESARI_VM_RECEIPT {\"schema_name\":\"macos-local-vm-resource-canary-guest-receipt\",\"schema_version\":\"1.0.0\",\"result\":\"%s\",\"attached_device_set_exact\":%s,\"host_canary_bytes_absent\":%s,\"host_paths_absent\":%s,\"host_process_invisible\":%s,\"memory_pressure_contained\":%s,\"memory_oom_kills\":\"%llu\",\"cpu_pressure_bounded\":%s,\"cpu_usage_usec\":\"%llu\",\"cpu_throttled_periods\":\"%llu\",\"pids_peak\":\"%llu\",\"job_cgroup_removed\":%s,\"source_retained\":false,\"authority_added\":false}\n",
           passed ? "passed" : "failed",
           exact_devices ? "true" : "false",
           no_canary_bytes ? "true" : "false",
           paths_absent ? "true" : "false",
           process_absent ? "true" : "false",
           memory_contained ? "true" : "false",
           (unsigned long long)oom_kills,
           cpu_bounded ? "true" : "false",
           (unsigned long long)cpu_usage,
           (unsigned long long)cpu_throttled,
           (unsigned long long)pids_peak,
           cgroup_removed ? "true" : "false");
    fflush(stdout);
    power_off();
}
