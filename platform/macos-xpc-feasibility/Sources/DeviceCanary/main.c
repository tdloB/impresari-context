// SPDX-License-Identifier: Apache-2.0
#include <errno.h>
#include <fcntl.h>
#include <signal.h>
#include <stddef.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

static int write_all(int descriptor, const char *bytes, size_t length) {
    size_t offset = 0;
    while (offset < length) {
        const ssize_t written = write(descriptor, bytes + offset, length - offset);
        if (written < 0) {
            if (errno == EINTR) {
                continue;
            }
            return -1;
        }
        offset += (size_t)written;
    }
    return 0;
}

int main(int argc, char **argv) {
    if (argc != 2) {
        return 2;
    }
    const int master = posix_openpt(O_RDWR | O_NOCTTY | O_CLOEXEC);
    if (master < 0 || grantpt(master) != 0 || unlockpt(master) != 0) {
        return 3;
    }
    const char *device_path = ptsname(master);
    if (device_path == NULL || strncmp(device_path, "/dev/ttys", 9) != 0) {
        return 4;
    }
    const int output = open(argv[1], O_WRONLY | O_CREAT | O_EXCL | O_CLOEXEC, 0600);
    if (output < 0) {
        return 5;
    }
    const size_t length = strlen(device_path);
    const int write_result = write_all(output, device_path, length);
    if (close(output) != 0 || write_result != 0) {
        return 6;
    }
    for (;;) {
        pause();
    }
}
