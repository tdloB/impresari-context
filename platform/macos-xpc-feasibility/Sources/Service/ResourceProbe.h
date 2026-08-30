// SPDX-License-Identifier: Apache-2.0
#ifndef IMPRESARI_RESOURCE_PROBE_H
#define IMPRESARI_RESOURCE_PROBE_H

#include <stdbool.h>
#include <stdint.h>

struct ImpresariDescendantProbe {
    bool fork_denied;
    bool spawn_denied;
};

struct ImpresariDescendantProbe impresari_probe_descendants(void);
uint64_t impresari_current_virtual_size(void);

#endif
