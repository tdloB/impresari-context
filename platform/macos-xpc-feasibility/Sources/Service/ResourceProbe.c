// SPDX-License-Identifier: Apache-2.0
#include "ResourceProbe.h"

#include <errno.h>
#include <mach/mach.h>
#include <spawn.h>
#include <sys/wait.h>
#include <unistd.h>

extern char **environ;

struct ImpresariDescendantProbe impresari_probe_descendants(void) {
    struct ImpresariDescendantProbe result = {false, false};

    errno = 0;
    pid_t child = fork();
    result.fork_denied = child == -1 && (errno == EAGAIN || errno == EPERM);
    if (child == 0) {
        _exit(0);
    }
    if (child > 0) {
        int status = 0;
        (void)waitpid(child, &status, 0);
    }

    pid_t spawned = 0;
    char executable[] = "/usr/bin/true";
    char *arguments[] = {executable, NULL};
    int spawn_result = posix_spawn(
        &spawned,
        executable,
        NULL,
        NULL,
        arguments,
        environ
    );
    result.spawn_denied = spawn_result == EAGAIN || spawn_result == EPERM;
    if (spawn_result == 0) {
        int status = 0;
        (void)waitpid(spawned, &status, 0);
    }

    return result;
}

uint64_t impresari_current_virtual_size(void) {
    mach_task_basic_info_data_t information;
    mach_msg_type_number_t count = MACH_TASK_BASIC_INFO_COUNT;
    kern_return_t result = task_info(
        mach_task_self(),
        MACH_TASK_BASIC_INFO,
        (task_info_t)&information,
        &count
    );
    return result == KERN_SUCCESS ? information.virtual_size : 0;
}
