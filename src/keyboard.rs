use evdev::uinput::{VirtualDevice, VirtualDeviceBuilder};
use evdev::{AttributeSet, Key};
use evdev::{EventType, InputEvent};
use std::thread::sleep;
use std::time::Duration;

use crate::EventHandler;

fn press(dev: &mut VirtualDevice, key: Key) {
    dev.emit(&[InputEvent::new(EventType::KEY, key.code(), 1)])
        .unwrap();

    sleep(Duration::from_millis(50));

    dev.emit(&[InputEvent::new(EventType::KEY, key.code(), 0)])
        .unwrap();
}

impl<'a> EventHandler<'a> {
    pub fn take_screenshot(&mut self) {
        press(&mut self.keyboard, Key::KEY_F5);
    }
}

impl<'a> EventHandler<'a> {
    pub fn clear_input(&mut self) {
        self.keyboard
            .emit(&[InputEvent::new(EventType::KEY, Key::KEY_LEFTCTRL.code(), 1)])
            .unwrap();
        self.keyboard
            .emit(&[InputEvent::new(EventType::KEY, Key::KEY_A.code(), 1)])
            .unwrap();

        sleep(Duration::from_millis(50));

        self.keyboard
            .emit(&[InputEvent::new(EventType::KEY, Key::KEY_A.code(), 0)])
            .unwrap();
        self.keyboard
            .emit(&[InputEvent::new(EventType::KEY, Key::KEY_LEFTCTRL.code(), 0)])
            .unwrap();

        self.keyboard
            .emit(&[InputEvent::new(EventType::KEY, Key::KEY_DELETE.code(), 1)])
            .unwrap();

        sleep(Duration::from_millis(50));

        self.keyboard
            .emit(&[InputEvent::new(EventType::KEY, Key::KEY_DELETE.code(), 0)])
            .unwrap();
    }
}

impl<'a> EventHandler<'a> {
    pub fn send_status(&mut self) {
        press(&mut self.keyboard, Key::KEY_S);
        press(&mut self.keyboard, Key::KEY_T);
        press(&mut self.keyboard, Key::KEY_A);
        press(&mut self.keyboard, Key::KEY_T);
        press(&mut self.keyboard, Key::KEY_U);
        press(&mut self.keyboard, Key::KEY_S);
        press(&mut self.keyboard, Key::KEY_ENTER);
    }
}

pub fn create_device() -> Result<VirtualDevice, std::io::Error> {
    let keys = &[
        Key::KEY_A,
        Key::KEY_S,
        Key::KEY_T,
        Key::KEY_U,
        Key::KEY_ENTER,
        Key::KEY_F5,
        Key::KEY_DELETE,
        Key::KEY_LEFTCTRL,
        Key::KEY_BACKSPACE,
    ]
    .iter()
    .collect::<AttributeSet<_>>();

    Ok(VirtualDeviceBuilder::new()?
        .name("EvType virtual keyboard")
        .with_keys(&keys)?
        .build()?)
}
