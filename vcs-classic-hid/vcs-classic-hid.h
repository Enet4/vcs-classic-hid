#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

/**
 * A representation of a game controller's stick position.
 */
typedef enum VcsClassicStickPosition {
  Center = 0,
  Up = 1,
  UpRight = 2,
  Right = 3,
  DownRight = 4,
  Down = 5,
  DownLeft = 6,
  Left = 7,
  UpLeft = 8,
} VcsClassicStickPosition;

/**
 * Opaque type representing the device
 */
typedef struct VcsClassicDevice VcsClassicDevice;

/**
 * Error code type
 */
typedef int VcsClassicHidError;

/**
 * A friendly representation of a game controller input state.
 */
typedef struct VcsClassicInputState {
  /**
   * The position of the stick
   */
  enum VcsClassicStickPosition stick_position;
  /**
   * Whether the main button is down
   */
  bool button_1;
  /**
   * Whether the secondary trigger is down
   */
  bool button_2;
  /**
   * Whether the back button is down
   */
  bool button_back;
  /**
   * Whether the menu/context button is down
   */
  bool button_menu;
  /**
   * Whether the Fuji (Atari) button is down
   */
  bool button_fuji;
  /**
   * The absolute position of the rotational paddle,
   * as a number between 0 and 1023
   */
  uint16_t roll;
} VcsClassicInputState;

/**
 * No error, operation successful
 */
#define VCS_CLASSIC_HID_ERROR_OK 0

/**
 * An HID error occurred
 */
#define VCS_CLASSIC_HID_ERROR_HID -2

/**
 * No device input was available on queue
 */
#define VCS_CLASSIC_HID_NO_INPUT 1

#ifdef __cplusplus
extern "C" {
#endif // __cplusplus

/**
 * Retrieve a string message of the last error occurred on this thread.
 *
 * **Safety:** Always discard the pointer (or never use it again)
 * before calling another function in this library.
 */
const char *vcs_classic_hid_last_error(void);

/**
 * Open access to the device.
 *
 * **Safety:** `p_device` must point to a valid mutable pointer.
 */
VcsClassicHidError vcs_classic_hid_open(struct VcsClassicDevice **p_device);

/**
 * Open access to the device.
 *
 * **Safety:** `p_device` must point to a valid mutable pointer.
 * and `path` must be a valid null terminated string.
 * The function does not check whether the device
 * behind the given path is actually the classic controller.
 */
VcsClassicHidError vcs_classic_hid_open_path(struct VcsClassicDevice **p_device, const char *path);

/**
 * Close an existing handle to the device.
 */
VcsClassicHidError vcs_classic_hid_close(struct VcsClassicDevice *p_device);

/**
 * Read a single HID report from the given device.
 */
VcsClassicHidError vcs_classic_hid_read(struct VcsClassicDevice *device,
                                        void *buf,
                                        size_t buf_len,
                                        size_t *report_len);

/**
 * Write an HID report from the given device.
 */
VcsClassicHidError vcs_classic_hid_write(struct VcsClassicDevice *device,
                                         const void *report,
                                         size_t report_len);

/**
 * Reset LED manipulation of the classic joystick device.
 */
VcsClassicHidError vcs_classic_hid_reset_leds(struct VcsClassicDevice *device);

/**
 * Process input reports in queue from the device
 * and write its current state.
 *
 * This function does not block.
 * If no input report was received,
 * the error code `vcs_classic_hid_NO_INPUT` is returned
 * and nothing is written to `state`.
 * When this happens, game loops should preferably assume
 * no changes occurred to the controller's input state.
 */
VcsClassicHidError vcs_classic_hid_process_input(struct VcsClassicDevice *device,
                                                 struct VcsClassicInputState *state);

/**
 * Initialize the input state object with blank information.
 */
void vcs_classic_hid_input_init(struct VcsClassicInputState *state);

#ifdef __cplusplus
} // extern "C"
#endif // __cplusplus
