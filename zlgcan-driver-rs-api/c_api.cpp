#include <iostream>
#include <thread>
#include "c_api.h"

int main() {
    uint32_t dev_type = 41;
    uint32_t dev_idx = 0;
    uint8_t send_chl = 0;
    uint8_t recv_chl = 1;

    const char *error = nullptr;
    const void *device = zlgcan_open(dev_type, dev_idx, nullptr, &error);
    if (nullptr == device) {
        printf("%s\n", error);
        return -1;
    }

    const char *dev_info = zlgcan_device_info(device, &error);
    printf("%s\n", dev_info);

    const CanChlCfgFactory *factory = zlgcan_cfg_factory_can();
    const void *cfg[2];
    for (auto & i : cfg) {
        const char *_error = nullptr;
        i = zlgcan_chl_cfg_can(factory, dev_type, 0, 0, 500000, &_error);
        if (i == nullptr) {
            printf("%s\n", _error);
            return -1;
        }
    }

    bool ret = zlgcan_init_can(device, cfg, 2, &error);
    if (!ret) {
        printf("%s\n", error);
        return -1;
    }

    for (int i = 0; i < 5; i++) {
        uint8_t data[5] = {1, 2, 3, 4, 5};
        struct CanMessage message = {
                .timestamp = 0,
                .arbitration_id = 0x123,
                .is_extended_id = false,
                .is_remote_frame = false,
                .is_error_frame = false,
                .channel = send_chl,
                .len = 5,
                .data = data,
                .is_fd = false,
                .is_rx = false,
                .bitrate_switch = false,
                .error_state_indicator = false
        };

        const char *_error = nullptr;
        zlgcan_send(device, message, &_error);
    }

    std::this_thread::sleep_for(std::chrono::milliseconds(100));

    const CanMessage *recv = nullptr;
    uint32_t count = zlgcan_recv(device, recv_chl, 0, &recv, &error);
    printf("received count: %d\n", count);
    for (int i = 0; i < count; i++) {
        auto message = recv[i];
        printf("received id: %d, length: %d\n", message.arbitration_id, message.len);
    }

    return 0;
}
