// X11 Keysym constants (from X11/keysymdef.h)
pub const XK_A: u32 = 0x0061;
pub const XK_B: u32 = 0x0062;
pub const XK_C: u32 = 0x0063;
pub const XK_D: u32 = 0x0064;
pub const XK_E: u32 = 0x0065;
pub const XK_F: u32 = 0x0066;
pub const XK_G: u32 = 0x0067;
pub const XK_H: u32 = 0x0068;
pub const XK_I: u32 = 0x0069;
pub const XK_J: u32 = 0x006a;
pub const XK_K: u32 = 0x006b;
pub const XK_L: u32 = 0x006c;
pub const XK_M: u32 = 0x006d;
pub const XK_N: u32 = 0x006e;
pub const XK_O: u32 = 0x006f;
pub const XK_P: u32 = 0x0070;
pub const XK_Q: u32 = 0x0071;
pub const XK_R: u32 = 0x0072;
pub const XK_S: u32 = 0x0073;
pub const XK_T: u32 = 0x0074;
pub const XK_U: u32 = 0x0075;
pub const XK_V: u32 = 0x0076;
pub const XK_W: u32 = 0x0077;
pub const XK_X: u32 = 0x0078;
pub const XK_Y: u32 = 0x0079;
pub const XK_Z: u32 = 0x007a;

pub const XK_0: u32 = 0x0030;
pub const XK_1: u32 = 0x0031;
pub const XK_2: u32 = 0x0032;
pub const XK_3: u32 = 0x0033;
pub const XK_4: u32 = 0x0034;
pub const XK_5: u32 = 0x0035;
pub const XK_6: u32 = 0x0036;
pub const XK_7: u32 = 0x0037;
pub const XK_8: u32 = 0x0038;
pub const XK_9: u32 = 0x0039;

pub const XK_F1: u32 = 0xffbe;
pub const XK_F2: u32 = 0xffbf;
pub const XK_F3: u32 = 0xffc0;
pub const XK_F4: u32 = 0xffc1;
pub const XK_F5: u32 = 0xffc2;
pub const XK_F6: u32 = 0xffc3;
pub const XK_F7: u32 = 0xffc4;
pub const XK_F8: u32 = 0xffc5;
pub const XK_F9: u32 = 0xffc6;
pub const XK_F10: u32 = 0xffc7;
pub const XK_F11: u32 = 0xffc8;
pub const XK_F12: u32 = 0xffc9;

pub const XK_SPACE: u32 = 0x0020;
pub const XK_RETURN: u32 = 0xff0d;
pub const XK_ESCAPE: u32 = 0xff1b;
pub const XK_TAB: u32 = 0xff09;
pub const XK_BACKSPACE: u32 = 0xff08;
pub const XK_DELETE: u32 = 0xffff;
pub const XK_INSERT: u32 = 0xff63;
pub const XK_HOME: u32 = 0xff50;
pub const XK_END: u32 = 0xff57;
pub const XK_PAGE_UP: u32 = 0xff55;
pub const XK_PAGE_DOWN: u32 = 0xff56;

pub const XK_UP: u32 = 0xff52;
pub const XK_DOWN: u32 = 0xff54;
pub const XK_LEFT: u32 = 0xff51;
pub const XK_RIGHT: u32 = 0xff53;

pub const XK_MINUS: u32 = 0x002d;
pub const XK_EQUAL: u32 = 0x003d;
pub const XK_BRACKETLEFT: u32 = 0x005b;
pub const XK_BRACKETRIGHT: u32 = 0x005d;
pub const XK_BACKSLASH: u32 = 0x005c;
pub const XK_SEMICOLON: u32 = 0x003b;
pub const XK_APOSTROPHE: u32 = 0x0027;
pub const XK_COMMA: u32 = 0x002c;
pub const XK_PERIOD: u32 = 0x002e;
pub const XK_SLASH: u32 = 0x002f;
pub const XK_GRAVE: u32 = 0x0060;

pub const XK_KP_0: u32 = 0xffb0;
pub const XK_KP_1: u32 = 0xffb1;
pub const XK_KP_2: u32 = 0xffb2;
pub const XK_KP_3: u32 = 0xffb3;
pub const XK_KP_4: u32 = 0xffb4;
pub const XK_KP_5: u32 = 0xffb5;
pub const XK_KP_6: u32 = 0xffb6;
pub const XK_KP_7: u32 = 0xffb7;
pub const XK_KP_8: u32 = 0xffb8;
pub const XK_KP_9: u32 = 0xffb9;
pub const XK_KP_ADD: u32 = 0xffab;
pub const XK_KP_SUBTRACT: u32 = 0xffad;
pub const XK_KP_MULTIPLY: u32 = 0xffaa;
pub const XK_KP_DIVIDE: u32 = 0xffaf;
pub const XK_KP_DECIMAL: u32 = 0xffae;
pub const XK_KP_ENTER: u32 = 0xff8d;

pub const XK_CAPS_LOCK: u32 = 0xffe5;
pub const XK_NUM_LOCK: u32 = 0xff7f;

// XF86 multimedia keys
pub const XF86XK_AUDIO_MUTE: u32 = 0x1008ff12;
pub const XF86XK_AUDIO_RAISE_VOLUME: u32 = 0x1008ff13;
pub const XF86XK_AUDIO_LOWER_VOLUME: u32 = 0x1008ff11;
pub const XF86XK_AUDIO_PLAY: u32 = 0x1008ff14;
pub const XF86XK_AUDIO_PREV: u32 = 0x1008ff16;
pub const XF86XK_AUDIO_NEXT: u32 = 0x1008ff17;
pub const XF86XK_MON_BRIGHTNESS_UP: u32 = 0x1008ff02;
pub const XF86XK_MON_BRIGHTNESS_DOWN: u32 = 0x1008ff03;

pub fn get_key_strings(state: &mut crate::state::State) -> Vec<String> {
    let mut keys: Vec<String> = vec![];
    keys.push(state.settings.bindings.launcher.clone());
    for k in &state.settings.bindings.swaps {
        keys.push(k.clone());
    }
    keys.push(state.settings.bindings.close_main.clone());
    keys.push(state.settings.bindings.fullscreen.clone());
    keys.push(state.settings.bindings.help.clone());
    keys.push(state.settings.bindings.terminal.clone());
    for k in &state.settings.bindings.workspaces {
        keys.push(k.clone());
    }
    keys.push(state.settings.bindings.monitor.clone());
    keys
}

pub fn parse_string(s: &str) -> Option<u32> {
    let s = s.to_lowercase();
    Some(match s.as_str() {
        // Letters
        "a" => XK_A,
        "b" => XK_B,
        "c" => XK_C,
        "d" => XK_D,
        "e" => XK_E,
        "f" => XK_F,
        "g" => XK_G,
        "h" => XK_H,
        "i" => XK_I,
        "j" => XK_J,
        "k" => XK_K,
        "l" => XK_L,
        "m" => XK_M,
        "n" => XK_N,
        "o" => XK_O,
        "p" => XK_P,
        "q" => XK_Q,
        "r" => XK_R,
        "s" => XK_S,
        "t" => XK_T,
        "u" => XK_U,
        "v" => XK_V,
        "w" => XK_W,
        "x" => XK_X,
        "y" => XK_Y,
        "z" => XK_Z,

        // Numbers
        "0" => XK_0,
        "1" => XK_1,
        "2" => XK_2,
        "3" => XK_3,
        "4" => XK_4,
        "5" => XK_5,
        "6" => XK_6,
        "7" => XK_7,
        "8" => XK_8,
        "9" => XK_9,

        // Function Keys
        "f1" => XK_F1,
        "f2" => XK_F2,
        "f3" => XK_F3,
        "f4" => XK_F4,
        "f5" => XK_F5,
        "f6" => XK_F6,
        "f7" => XK_F7,
        "f8" => XK_F8,
        "f9" => XK_F9,
        "f10" => XK_F10,
        "f11" => XK_F11,
        "f12" => XK_F12,

        // Special Keys
        "spc" | "space" => XK_SPACE,
        "ret" | "enter" => XK_RETURN,
        "esc" => XK_ESCAPE,
        "tab" => XK_TAB,
        "bsp" | "backspace" => XK_BACKSPACE,
        "del" => XK_DELETE,
        "ins" => XK_INSERT,
        "home" => XK_HOME,
        "end" => XK_END,
        "pgup" => XK_PAGE_UP,
        "pgdn" => XK_PAGE_DOWN,

        // Arrow Keys
        "up" => XK_UP,
        "down" => XK_DOWN,
        "left" => XK_LEFT,
        "right" => XK_RIGHT,

        // Punctuation
        "-" => XK_MINUS,
        "=" => XK_EQUAL,
        "[" => XK_BRACKETLEFT,
        "]" => XK_BRACKETRIGHT,
        "\\" => XK_BACKSLASH,
        ";" => XK_SEMICOLON,
        "'" => XK_APOSTROPHE,
        "," => XK_COMMA,
        "." => XK_PERIOD,
        "/" => XK_SLASH,
        "`" => XK_GRAVE,

        // Keypad
        "kp0" => XK_KP_0,
        "kp1" => XK_KP_1,
        "kp2" => XK_KP_2,
        "kp3" => XK_KP_3,
        "kp4" => XK_KP_4,
        "kp5" => XK_KP_5,
        "kp6" => XK_KP_6,
        "kp7" => XK_KP_7,
        "kp8" => XK_KP_8,
        "kp9" => XK_KP_9,
        "kp+" => XK_KP_ADD,
        "kp-" => XK_KP_SUBTRACT,
        "kp*" => XK_KP_MULTIPLY,
        "kp/" => XK_KP_DIVIDE,
        "kp." => XK_KP_DECIMAL,
        "kpret" => XK_KP_ENTER,

        // Locks
        "caps" => XK_CAPS_LOCK,
        "num" => XK_NUM_LOCK,

        // Multimedia
        "mute" => XF86XK_AUDIO_MUTE,
        "volu" => XF86XK_AUDIO_RAISE_VOLUME,
        "vold" => XF86XK_AUDIO_LOWER_VOLUME,
        "play" => XF86XK_AUDIO_PLAY,
        "prev" => XF86XK_AUDIO_PREV,
        "next" => XF86XK_AUDIO_NEXT,
        "briu" => XF86XK_MON_BRIGHTNESS_UP,
        "brid" => XF86XK_MON_BRIGHTNESS_DOWN,

        _ => return None,
    })
}
