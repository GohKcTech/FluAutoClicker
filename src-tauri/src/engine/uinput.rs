#[cfg(target_os = "linux")]
use evdev::uinput::{VirtualDevice, VirtualDeviceBuilder};
#[cfg(target_os = "linux")]
use evdev::{AbsInfo, AbsoluteAxisType, AttributeSet, Key, RelativeAxisType, UinputAbsSetup};

#[cfg(target_os = "linux")]
pub fn setup_uinput() -> Option<VirtualDevice> {
    let mut keys = AttributeSet::<Key>::new();
    keys.insert(Key::BTN_LEFT);
    keys.insert(Key::BTN_RIGHT);
    keys.insert(Key::BTN_MIDDLE);
    keys.insert(Key::BTN_SIDE);
    keys.insert(Key::BTN_EXTRA);

    let mut rels = AttributeSet::<RelativeAxisType>::new();
    rels.insert(RelativeAxisType::REL_X);
    rels.insert(RelativeAxisType::REL_Y);

    let abs_info = AbsInfo::new(0, 0, 32767, 0, 0, 0);
    let abs_x = UinputAbsSetup::new(AbsoluteAxisType::ABS_X, abs_info);
    let abs_y = UinputAbsSetup::new(AbsoluteAxisType::ABS_Y, abs_info);

    VirtualDeviceBuilder::new()
        .ok()?
        .name("FluAutoClicker Virtual Mouse")
        .with_keys(&keys)
        .ok()?
        .with_absolute_axis(&abs_x)
        .ok()?
        .with_absolute_axis(&abs_y)
        .ok()?
        .with_relative_axes(&rels)
        .ok()?
        .build()
        .ok()
}
