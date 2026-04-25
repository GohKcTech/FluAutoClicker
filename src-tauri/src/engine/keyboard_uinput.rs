#[cfg(target_os = "linux")]
use evdev::uinput::{VirtualDevice, VirtualDeviceBuilder};
#[cfg(target_os = "linux")]
use evdev::AttributeSet;
#[cfg(target_os = "linux")]
use evdev::Key;

#[cfg(target_os = "linux")]
pub fn setup_keyboard_uinput() -> Option<VirtualDevice> {
    let mut keys = AttributeSet::<Key>::new();

    for i in 1..=12 {
        match i {
            1 => keys.insert(Key::KEY_F1),
            2 => keys.insert(Key::KEY_F2),
            3 => keys.insert(Key::KEY_F3),
            4 => keys.insert(Key::KEY_F4),
            5 => keys.insert(Key::KEY_F5),
            6 => keys.insert(Key::KEY_F6),
            7 => keys.insert(Key::KEY_F7),
            8 => keys.insert(Key::KEY_F8),
            9 => keys.insert(Key::KEY_F9),
            10 => keys.insert(Key::KEY_F10),
            11 => keys.insert(Key::KEY_F11),
            12 => keys.insert(Key::KEY_F12),
            _ => {}
        }
    }

    keys.insert(Key::KEY_ESC);

    keys.insert(Key::KEY_0);
    keys.insert(Key::KEY_1);
    keys.insert(Key::KEY_2);
    keys.insert(Key::KEY_3);
    keys.insert(Key::KEY_4);
    keys.insert(Key::KEY_5);
    keys.insert(Key::KEY_6);
    keys.insert(Key::KEY_7);
    keys.insert(Key::KEY_8);
    keys.insert(Key::KEY_9);

    keys.insert(Key::KEY_A);
    keys.insert(Key::KEY_B);
    keys.insert(Key::KEY_C);
    keys.insert(Key::KEY_D);
    keys.insert(Key::KEY_E);
    keys.insert(Key::KEY_F);
    keys.insert(Key::KEY_G);
    keys.insert(Key::KEY_H);
    keys.insert(Key::KEY_I);
    keys.insert(Key::KEY_J);
    keys.insert(Key::KEY_K);
    keys.insert(Key::KEY_L);
    keys.insert(Key::KEY_M);
    keys.insert(Key::KEY_N);
    keys.insert(Key::KEY_O);
    keys.insert(Key::KEY_P);
    keys.insert(Key::KEY_Q);
    keys.insert(Key::KEY_R);
    keys.insert(Key::KEY_S);
    keys.insert(Key::KEY_T);
    keys.insert(Key::KEY_U);
    keys.insert(Key::KEY_V);
    keys.insert(Key::KEY_W);
    keys.insert(Key::KEY_X);
    keys.insert(Key::KEY_Y);
    keys.insert(Key::KEY_Z);

    keys.insert(Key::KEY_LEFTCTRL);
    keys.insert(Key::KEY_RIGHTCTRL);
    keys.insert(Key::KEY_LEFTSHIFT);
    keys.insert(Key::KEY_RIGHTSHIFT);
    keys.insert(Key::KEY_LEFTALT);
    keys.insert(Key::KEY_RIGHTALT);
    keys.insert(Key::KEY_LEFTMETA);
    keys.insert(Key::KEY_RIGHTMETA);

    keys.insert(Key::KEY_SPACE);
    keys.insert(Key::KEY_ENTER);
    keys.insert(Key::KEY_TAB);
    keys.insert(Key::KEY_BACKSPACE);
    keys.insert(Key::KEY_DELETE);
    keys.insert(Key::KEY_INSERT);
    keys.insert(Key::KEY_HOME);
    keys.insert(Key::KEY_END);
    keys.insert(Key::KEY_PAGEUP);
    keys.insert(Key::KEY_PAGEDOWN);

    keys.insert(Key::KEY_UP);
    keys.insert(Key::KEY_DOWN);
    keys.insert(Key::KEY_LEFT);
    keys.insert(Key::KEY_RIGHT);

    keys.insert(Key::KEY_KP0);
    keys.insert(Key::KEY_KP1);
    keys.insert(Key::KEY_KP2);
    keys.insert(Key::KEY_KP3);
    keys.insert(Key::KEY_KP4);
    keys.insert(Key::KEY_KP5);
    keys.insert(Key::KEY_KP6);
    keys.insert(Key::KEY_KP7);
    keys.insert(Key::KEY_KP8);
    keys.insert(Key::KEY_KP9);
    keys.insert(Key::KEY_KPENTER);
    keys.insert(Key::KEY_KPPLUS);
    keys.insert(Key::KEY_KPMINUS);
    keys.insert(Key::KEY_KPASTERISK);
    keys.insert(Key::KEY_KPDOT);
    keys.insert(Key::KEY_KPSLASH);
    keys.insert(Key::KEY_NUMLOCK);

    keys.insert(Key::KEY_GRAVE);
    keys.insert(Key::KEY_MINUS);
    keys.insert(Key::KEY_EQUAL);
    keys.insert(Key::KEY_LEFTBRACE);
    keys.insert(Key::KEY_RIGHTBRACE);
    keys.insert(Key::KEY_BACKSLASH);
    keys.insert(Key::KEY_SEMICOLON);
    keys.insert(Key::KEY_APOSTROPHE);
    keys.insert(Key::KEY_COMMA);
    keys.insert(Key::KEY_DOT);
    keys.insert(Key::KEY_SLASH);

    VirtualDeviceBuilder::new()
        .ok()?
        .name("FluAutoClicker Virtual Keyboard")
        .with_keys(&keys)
        .ok()?
        .build()
        .ok()
}
