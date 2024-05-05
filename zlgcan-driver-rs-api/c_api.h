#include <cstdarg>
#include <cstdint>
#include <cstdlib>
#include <ostream>
#include <new>

struct CanChlCfgFactory {
    uintptr_t data;
};

struct DeriveInfo {
    bool canfd;
    uint8_t channels;
};

struct CanMessage {
    uint64_t timestamp;
    uint32_t arbitration_id;
    bool is_extended_id;
    bool is_remote_frame;
    bool is_error_frame;
    uint8_t channel;
    uint8_t len;
    const uint8_t *data;
    bool is_fd;
    bool is_rx;
    bool bitrate_switch;
    bool error_state_indicator;
};

struct ZCanChlCfg {
    uint32_t dev_type;
    uint8_t chl_type;
    uint8_t chl_mode;
    uint32_t bitrate;
    uint8_t *filter;
    uint32_t *dbitrate;
    bool *resistance;
    uint32_t *acc_code;
    uint32_t *acc_mask;
    uint32_t *brp;
};

extern "C" {

const CanChlCfgFactory *zlgcan_cfg_factory_can(const char **error);

const void *zlgcan_chl_cfg_can(const CanChlCfgFactory *factory,
                               struct ZCanChlCfg cfg,
                               const char **error);

const void *zlgcan_open(uint32_t dev_type,
                        uint32_t dev_idx,
                        const DeriveInfo *derive,
                        const char **error);

bool zlgcan_init_can(const void *device, const void *cfg, uintptr_t len, const char **error);

const char *zlgcan_device_info(const void *device, const char **error);

bool zlgcan_clear_can_buffer(const void *device, uint8_t channel, const char **error);

bool zlgcan_send(const void *device, CanMessage msg, const char **error);

uint32_t zlgcan_recv(const void *device,
                     uint8_t channel,
                     uint32_t timeout,
                     const CanMessage **buffer,
                     const char **error);

void zlgcan_close(const void *device);

} // extern "C"
