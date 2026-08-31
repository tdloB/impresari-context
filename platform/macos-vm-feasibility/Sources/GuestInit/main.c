// SPDX-License-Identifier: Apache-2.0
#define _GNU_SOURCE

#include <dirent.h>
#include <errno.h>
#include <fcntl.h>
#include <linux/reboot.h>
#include <sched.h>
#include <signal.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>
#include <sys/mount.h>
#include <sys/reboot.h>
#include <sys/stat.h>
#include <sys/syscall.h>
#include <sys/types.h>
#include <sys/wait.h>
#include <unistd.h>

#define INPUT_BYTES 4096
#define SCRATCH_BYTES 1048576
#define PAGE_BYTES 4096

static const char input_prefix[] = "IMPRESARI_VM_INPUT_V1\nsynthetic-only\n";
static const char marker[] = "IMPRESARI_JOB_MARKER_V1";

enum scenario {
    SCENARIO_SUCCESS,
    SCENARIO_MALFORMED_RESULT,
    SCENARIO_OUTPUT_FLOOD,
    SCENARIO_TIMEOUT,
    SCENARIO_DESCENDANT_TIMEOUT,
    SCENARIO_EARLY_EXIT
};

static enum scenario selected_scenario(void) {
    mkdir("/proc", 0555);
    if (mount("proc", "/proc", "proc", MS_NOSUID | MS_NODEV | MS_NOEXEC, "") != 0 && errno != EBUSY) {
        return SCENARIO_EARLY_EXIT;
    }
    char command_line[512];
    int fd = open("/proc/cmdline", O_RDONLY | O_CLOEXEC);
    if (fd < 0) {
        return SCENARIO_EARLY_EXIT;
    }
    ssize_t amount = read(fd, command_line, sizeof(command_line) - 1);
    close(fd);
    if (amount <= 0) {
        return SCENARIO_EARLY_EXIT;
    }
    command_line[amount] = '\0';
    if (strstr(command_line, " impresari.mode=malformed-result") != NULL) {
        return SCENARIO_MALFORMED_RESULT;
    }
    if (strstr(command_line, " impresari.mode=output-flood") != NULL) {
        return SCENARIO_OUTPUT_FLOOD;
    }
    if (strstr(command_line, " impresari.mode=timeout") != NULL) {
        return SCENARIO_TIMEOUT;
    }
    if (strstr(command_line, " impresari.mode=descendant-timeout") != NULL) {
        return SCENARIO_DESCENDANT_TIMEOUT;
    }
    if (strstr(command_line, " impresari.mode=early-exit") != NULL) {
        return SCENARIO_EARLY_EXIT;
    }
    return SCENARIO_SUCCESS;
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

static bool input_matches(void) {
    unsigned char buffer[INPUT_BYTES];
    int fd = open("/dev/vda", O_RDONLY | O_CLOEXEC);
    if (fd < 0) {
        return false;
    }
    ssize_t amount = pread(fd, buffer, sizeof(buffer), 0);
    close(fd);
    if (amount != (ssize_t)sizeof(buffer) ||
        memcmp(buffer, input_prefix, sizeof(input_prefix) - 1) != 0) {
        return false;
    }
    for (size_t index = sizeof(input_prefix) - 1; index < sizeof(buffer); index++) {
        if (buffer[index] != 0) {
            return false;
        }
    }
    return true;
}

static bool input_write_denied(void) {
    int fd = open("/dev/vda", O_RDWR | O_CLOEXEC);
    if (fd < 0) {
        return errno == EROFS || errno == EACCES || errno == EPERM;
    }
    unsigned char byte = 1;
    ssize_t amount = pwrite(fd, &byte, 1, 0);
    int saved_errno = errno;
    close(fd);
    return amount < 0 && (saved_errno == EROFS || saved_errno == EACCES || saved_errno == EPERM);
}

static bool scratch_initially_clean(int fd) {
    unsigned char buffer[sizeof(marker) - 1];
    if (pread(fd, buffer, sizeof(buffer), 0) != (ssize_t)sizeof(buffer)) {
        return false;
    }
    for (size_t index = 0; index < sizeof(buffer); index++) {
        if (buffer[index] != 0) {
            return false;
        }
    }
    return true;
}

static bool fill_scratch_and_verify_bound(int fd) {
    unsigned char page[PAGE_BYTES];
    memset(page, 0xA5, sizeof(page));
    for (off_t offset = 0; offset < SCRATCH_BYTES; offset += PAGE_BYTES) {
        if (pwrite(fd, page, sizeof(page), offset) != (ssize_t)sizeof(page)) {
            return false;
        }
    }
    unsigned char byte = 0x5A;
    ssize_t beyond = pwrite(fd, &byte, 1, SCRATCH_BYTES);
    if (beyond > 0) {
        return false;
    }
    if (pwrite(fd, marker, sizeof(marker) - 1, 0) != (ssize_t)(sizeof(marker) - 1)) {
        return false;
    }
    return fsync(fd) == 0;
}

static bool network_device_absent(void) {
    if (mount("sysfs", "/sys", "sysfs", MS_NOSUID | MS_NODEV | MS_NOEXEC, "") != 0 && errno != EBUSY) {
        return false;
    }
    DIR *directory = opendir("/sys/class/net");
    if (directory == NULL) {
        return false;
    }
    bool only_loopback = true;
    struct dirent *entry;
    while ((entry = readdir(directory)) != NULL) {
        if (strcmp(entry->d_name, ".") != 0 && strcmp(entry->d_name, "..") != 0 &&
            strcmp(entry->d_name, "lo") != 0) {
            only_loopback = false;
        }
    }
    closedir(directory);
    return only_loopback;
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
    mkdir("/sys", 0555);
    bool dev_ready = mount("devtmpfs", "/dev", "devtmpfs", MS_NOSUID | MS_NOEXEC, "mode=0755") == 0 || errno == EBUSY;
    bool driver_loaded = dev_ready && load_block_driver();
    bool disks_ready = driver_loaded && wait_for_path("/dev/vda") && wait_for_path("/dev/vdb");
    bool exact_input = disks_ready && input_matches();
    bool read_only_input = disks_ready && input_write_denied();
    bool clean_scratch = false;
    bool bounded_scratch = false;
    if (disks_ready) {
        int scratch = open("/dev/vdb", O_RDWR | O_CLOEXEC);
        if (scratch >= 0) {
            clean_scratch = scratch_initially_clean(scratch);
            bounded_scratch = fill_scratch_and_verify_bound(scratch);
            close(scratch);
        }
    }
    bool no_network_device = network_device_absent();
    bool passed = exact_input && read_only_input && clean_scratch && bounded_scratch && no_network_device;

    enum scenario scenario = selected_scenario();
    if (scenario == SCENARIO_MALFORMED_RESULT) {
        printf("IMPRESARI_VM_RECEIPT {malformed\n");
        fflush(stdout);
        power_off();
    }
    if (scenario == SCENARIO_OUTPUT_FLOOD) {
        for (size_t index = 0; index < 131072; index++) {
            putchar('X');
        }
        putchar('\n');
        fflush(stdout);
        power_off();
    }
    if (scenario == SCENARIO_TIMEOUT) {
        for (;;) {
            pause();
        }
    }
    if (scenario == SCENARIO_DESCENDANT_TIMEOUT) {
        pid_t child = fork();
        if (child < 0) {
            power_off();
        }
        for (;;) {
            pause();
        }
    }
    if (scenario == SCENARIO_EARLY_EXIT) {
        power_off();
    }

    printf("IMPRESARI_VM_RECEIPT {\"schema_name\":\"macos-local-vm-guest-receipt\",\"schema_version\":\"1.0.0\",\"result\":\"%s\",\"exact_input_verified\":%s,\"read_only_input_verified\":%s,\"scratch_initially_clean\":%s,\"scratch_capacity_verified\":%s,\"network_device_absent\":%s,\"source_retained\":false,\"authority_added\":false}\n",
           passed ? "passed" : "failed",
           exact_input ? "true" : "false",
           read_only_input ? "true" : "false",
           clean_scratch ? "true" : "false",
           bounded_scratch ? "true" : "false",
           no_network_device ? "true" : "false");
    fflush(stdout);
    power_off();
}
