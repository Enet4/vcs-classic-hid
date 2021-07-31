#include <stdio.h>
#include <unistd.h>
#include <stdlib.h>
#include <time.h>
#include "../vcs-classic-hid.h"

#define TRY(C)                                                     \
    {                                                              \
        if (C) {                                                   \
            fprintf(stderr, "%s\n", vcs_classic_hid_last_error()); \
            exit(-2);                                              \
        }                                                          \
    }

int main() {
    int e;
    VcsClassicDevice* device = NULL;

    struct timespec sleep_time;
    sleep_time.tv_sec = 0;
//    sleep_time.tv_nsec = 25000000l;
    sleep_time.tv_nsec = 50000000l;
    struct timespec time_remaining;
    time_remaining.tv_sec = 0;
    time_remaining.tv_nsec = 0;

    TRY(vcs_classic_hid_open(&device));

    printf("Successfully opened classic device.\n");

    VcsClassicInputState state;
    vcs_classic_hid_input_init(&state);

    unsigned char led_report[28] = {
        2, // LED report
        25, // number of LEDs to change, must be odd
        128, // Fuji LED blinking at half intensity
        0, // LED #0 in the ring, and so on
    };
    unsigned char intensity = 0;
    for (unsigned int tick = 0 ; tick < 500; tick++) {
        printf("Tick #%02d: ", tick);
        fflush(stdout);

        // check input
        e = vcs_classic_hid_process_input(device, &state);
        if (e != 0 && e != VCS_CLASSIC_HID_NO_INPUT) {
            TRY(e);
        }
        printf("got input; ");
        fflush(stdout);

        // detect fuji button press to exit
        if (state.button_fuji) {
            printf("\nFuji button pressed!\n");
            break;
        }

        for (unsigned int i = 3; i < 27; i++) {
            led_report[i] = intensity;
        }

        // send report
        TRY(vcs_classic_hid_write(device, led_report, 28));
        printf("LED report sent");
        fflush(stdout);

        intensity += 8;
        nanosleep(&sleep_time, &time_remaining);
        printf("\r                                       \r");
        fflush(stdout);
    }
    printf("\n");

    // reset LED manipulation
    // (and wait a bit to give it time to fulfill)
    TRY(vcs_classic_hid_reset_leds(device));
    nanosleep(&sleep_time, &time_remaining);

    printf("Closing device.\n");
    TRY(vcs_classic_hid_close(device));
}