//! FFI interface for `vcs-classic-hid`

use std::{cell::{Cell, RefCell}, ffi::{CStr, CString}, fmt::Display};
use libc::{c_char, c_int, c_void, size_t};

/// Error code type
pub type VcsClassicHidError = c_int;

/// No error, operation successful
pub const VCS_CLASSIC_HID_ERROR_OK: VcsClassicHidError = 0;
/// An HID error occurred
pub const VCS_CLASSIC_HID_ERROR_HID: VcsClassicHidError = -2;
/// No device input was available on queue
pub const VCS_CLASSIC_HID_NO_INPUT: VcsClassicHidError = 1;

std::thread_local! {
    static LAST_ERROR_CODE: Cell<VcsClassicHidError> = Cell::new(0);
    static LAST_ERROR_MSG: RefCell<CString> = RefCell::new(CString::default());
}

/// Opaque type representing the device
pub struct VcsClassicDevice {
    _opaque: [(); 0],
}

/// A representation of a game controller's stick position.
#[repr(C)]
#[derive(Debug, Copy, Clone, Eq, Hash, PartialEq)]
pub enum VcsClassicStickPosition {
    Center = 0,
    Up = 1,
    UpRight = 2,
    Right = 3,
    DownRight = 4,
    Down = 5,
    DownLeft = 6,
    Left = 7,
    UpLeft = 8,
}

impl Default for VcsClassicStickPosition {
    fn default() -> Self {
        VcsClassicStickPosition::Center
    }
}

impl From<crate::input::StickPosition> for VcsClassicStickPosition {
    fn from(value: crate::input::StickPosition) -> Self {
        match value {
            crate::StickPosition::Center => VcsClassicStickPosition::Center,
            crate::StickPosition::Up => VcsClassicStickPosition::Up,
            crate::StickPosition::UpRight => VcsClassicStickPosition::UpRight,
            crate::StickPosition::Right => VcsClassicStickPosition::Right,
            crate::StickPosition::DownRight => VcsClassicStickPosition::DownRight,
            crate::StickPosition::Down => VcsClassicStickPosition::Down,
            crate::StickPosition::DownLeft => VcsClassicStickPosition::DownLeft,
            crate::StickPosition::Left => VcsClassicStickPosition::Left,
            crate::StickPosition::UpLeft => VcsClassicStickPosition::UpLeft,
        }
    }    
}

/// A friendly representation of a game controller input state.
#[derive(Debug, Default, Copy, Clone, PartialEq)]
#[repr(C)]
pub struct VcsClassicInputState {
    /// The position of the stick
    pub stick_position: VcsClassicStickPosition,
    /// Whether the main button is down
    pub button_1: bool,
    /// Whether the secondary trigger is down
    pub button_2: bool,
    /// Whether the back button is down
    pub button_back: bool,
    /// Whether the menu/context button is down
    pub button_menu: bool,
    /// Whether the Fuji (Atari) button is down
    pub button_fuji: bool,
    /// The absolute position of the rotational paddle,
    /// as a number between 0 and 1023
    pub roll: u16,
}

impl From<crate::input::State> for VcsClassicInputState {
    fn from(value: crate::input::State) -> Self {
        VcsClassicInputState {
            stick_position: value.stick_position.into(),
            button_1: value.button_1,
            button_2: value.button_2,
            button_back: value.button_back,
            button_menu: value.button_menu,
            button_fuji: value.button_fuji,
            roll: value.roll,
        }
    }    
}


#[inline]
fn err_to_code<T>(error: T, code: i32) -> VcsClassicHidError
where
    T: Display,
{
    LAST_ERROR_CODE.with(|e| {
        e.set(code);
    });
    LAST_ERROR_MSG.with(|e| {
        *e.borrow_mut() = CString::new(error.to_string().as_bytes())
            .unwrap_or_else(|_| CString::new(&b"ERROR"[..]).unwrap());
    });
    code
}

#[inline]
fn result_to_code<T, E>(result: Result<T, E>, code: i32) -> VcsClassicHidError
where
    E: Display,
{
    match result {
        Ok(_) => 0,
        Err(error) => err_to_code(error, code),
    }
}

/// Retrieve a string message of the last error occurred on this thread.
///
/// **Safety:** Always discard the pointer (or never use it again)
/// before calling another function in this library.
#[no_mangle]
pub unsafe extern "C" fn vcs_classic_hid_last_error() -> *const std::os::raw::c_char {
    LAST_ERROR_MSG.with(|e| e.borrow().as_c_str().as_ptr() as *const c_char)
}

/// Open access to the device.
///
/// **Safety:** `p_device` must point to a valid mutable pointer.
#[no_mangle]
pub unsafe extern "C" fn vcs_classic_hid_open(p_device: *mut *mut VcsClassicDevice) -> VcsClassicHidError {
    match crate::open() {
        Ok(device) => {
            let device = Box::new(device);
            let p = Box::into_raw(device);
            *p_device = p as *mut _;
            0
        }
        Err(e) => err_to_code(e, VCS_CLASSIC_HID_ERROR_HID),
    }
}

/// Open access to the device.
///
/// **Safety:** `p_device` must point to a valid mutable pointer.
/// and `path` must be a valid null terminated string.
/// The function does not check whether the device
/// behind the given path is actually the classic controller.
#[no_mangle]
pub unsafe extern "C" fn vcs_classic_hid_open_path(p_device: *mut *mut VcsClassicDevice, path: *const c_char) -> VcsClassicHidError {
    let path = CStr::from_ptr(path);

    match crate::open_path(path) {
        Ok(device) => {
            let device = Box::new(device);
            let p = Box::into_raw(device);
            *p_device = p as *mut _;
            0
        }
        Err(e) => err_to_code(e, VCS_CLASSIC_HID_ERROR_HID),
    }
}

/// Close an existing handle to the device.
#[no_mangle]
pub unsafe extern "C" fn vcs_classic_hid_close(p_device: *mut VcsClassicDevice) -> VcsClassicHidError {
    let _ = Box::from_raw(p_device as *mut _);
    0
}

/// Read a single HID report from the given device.
#[no_mangle]
pub unsafe extern "C" fn vcs_classic_hid_read(device: *mut VcsClassicDevice, buf: *mut c_void, buf_len: size_t, report_len: *mut size_t) -> VcsClassicHidError {
    let device: &mut _ = (device as *mut crate::hidapi::HidDevice).as_mut().unwrap();

    let buf = std::slice::from_raw_parts_mut(buf as *mut u8, buf_len);

    match crate::Device::read(device, buf) {
        Ok(l) => {
            *report_len = l;
            0
        },
        Err(e) => err_to_code(e, VCS_CLASSIC_HID_ERROR_HID),
    }
}

/// Write an HID report from the given device.
#[no_mangle]
pub unsafe extern "C" fn vcs_classic_hid_write(device: *mut VcsClassicDevice, report: *const c_void, report_len: size_t) -> VcsClassicHidError {
    let device: &mut _ = (device as *mut crate::hidapi::HidDevice).as_mut().unwrap();

    let buf = std::slice::from_raw_parts(report as *const u8, report_len);

    result_to_code(crate::Device::write(device, buf), VCS_CLASSIC_HID_ERROR_HID)
}

/// Reset LED manipulation of the classic joystick device.
#[no_mangle]
pub unsafe extern "C" fn vcs_classic_hid_reset_leds(device: *mut VcsClassicDevice) -> VcsClassicHidError {
    let device: &mut _ = (device as *mut crate::hidapi::HidDevice).as_mut().unwrap();

    use crate::Device;
    result_to_code(device.reset_leds(), VCS_CLASSIC_HID_ERROR_HID)
}

/// Process input reports in queue from the device
/// and write its current state.
///
/// This function does not block.
/// If no input report was received,
/// the error code `vcs_classic_hid_NO_INPUT` is returned
/// and nothing is written to `state`.
/// When this happens, game loops should preferably assume
/// no changes occurred to the controller's input state.
#[no_mangle]
pub unsafe extern "C" fn vcs_classic_hid_process_input(device: *mut VcsClassicDevice, state: *mut VcsClassicInputState) -> VcsClassicHidError {
    let device: &mut _ = (device as *mut crate::hidapi::HidDevice).as_mut().unwrap();

    let s = match crate::process_input(device) {
        Ok(Some(s)) => s,
        Ok(None) => return err_to_code("No input", VCS_CLASSIC_HID_NO_INPUT),
        Err(e) => return err_to_code(e, VCS_CLASSIC_HID_ERROR_HID),
    };

    std::ptr::write(state, VcsClassicInputState::from(s));
    VCS_CLASSIC_HID_ERROR_OK
}

/// Initialize the input state object with blank information.
#[no_mangle]
pub unsafe extern "C" fn vcs_classic_hid_input_init(state: *mut VcsClassicInputState) {
    std::ptr::write(state, VcsClassicInputState::default());
}
