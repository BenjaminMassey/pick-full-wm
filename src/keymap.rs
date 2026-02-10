pub fn get_key_strings(state: &mut crate::state::State) -> Vec<String> {
    let mut keys: Vec<String> = vec![];
    for (k, _) in &state.settings.bindings.functions {
        keys.push(k.clone());
    }
    for k in &state.settings.bindings.swaps {
        keys.push(k.clone());
    }
    keys.push(state.settings.bindings.close_main.clone());
    keys.push(state.settings.bindings.fullscreen.clone());
    keys.push(state.settings.bindings.help.clone());
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
        "a" => 0x0061,
        "b" => 0x0062,
        "c" => 0x0063,
        "d" => 0x0064,
        "e" => 0x0065,
        "f" => 0x0066,
        "g" => 0x0067,
        "h" => 0x0068,
        "i" => 0x0069,
        "j" => 0x006a,
        "k" => 0x006b,
        "l" => 0x006c,
        "m" => 0x006d,
        "n" => 0x006e,
        "o" => 0x006f,
        "p" => 0x0070,
        "q" => 0x0071,
        "r" => 0x0072,
        "s" => 0x0073,
        "t" => 0x0074,
        "u" => 0x0075,
        "v" => 0x0076,
        "w" => 0x0077,
        "x" => 0x0078,
        "y" => 0x0079,
        "z" => 0x007a,

        // Numbers
        "0" => 0x0030,
        "1" => 0x0031,
        "2" => 0x0032,
        "3" => 0x0033,
        "4" => 0x0034,
        "5" => 0x0035,
        "6" => 0x0036,
        "7" => 0x0037,
        "8" => 0x0038,
        "9" => 0x0039,

        // Function Keys
        "f1" => 0xffbe,
        "f2" => 0xffbf,
        "f3" => 0xffc0,
        "f4" => 0xffc1,
        "f5" => 0xffc2,
        "f6" => 0xffc3,
        "f7" => 0xffc4,
        "f8" => 0xffc5,
        "f9" => 0xffc6,
        "f10" => 0xffc7,
        "f11" => 0xffc8,
        "f12" => 0xffc9,

        // Special Keys
        "spc" | "space" => 0x0020,
        "ret" | "enter" => 0xff0d,
        "esc" => 0xff1b,
        "tab" => 0xff09,
        "bsp" | "backspace" => 0xff08,
        "del" => 0xffff,
        "ins" => 0xff63,
        "home" => 0xff50,
        "end" => 0xff57,
        "pgup" => 0xff55,
        "pgdn" => 0xff56,

        // Arrow Keys
        "up" => 0xff52,
        "down" => 0xff54,
        "left" => 0xff51,
        "right" => 0xff53,

        // Punctuation
        "-" => 0x002d,
        "=" => 0x003d,
        "[" => 0x005b,
        "]" => 0x005d,
        "\\" => 0x005c,
        ";" => 0x003b,
        "'" => 0x0027,
        "," => 0x002c,
        "." => 0x002e,
        "/" => 0x002f,
        "`" => 0x0060,

        // Keypad
        "kp0" => 0xffb0,
        "kp1" => 0xffb1,
        "kp2" => 0xffb2,
        "kp3" => 0xffb3,
        "kp4" => 0xffb4,
        "kp5" => 0xffb5,
        "kp6" => 0xffb6,
        "kp7" => 0xffb7,
        "kp8" => 0xffb8,
        "kp9" => 0xffb9,
        "kp+" => 0xffab,
        "kp-" => 0xffad,
        "kp*" => 0xffaa,
        "kp/" => 0xffaf,
        "kp." => 0xffae,
        "kpret" => 0xff8d,

        // Locks
        "caps" => 0xffe5,
        "num" => 0xff7f,

        // Multimedia
        "mute" => 0x1008ff12,
        "volu" => 0x1008ff13,
        "vold" => 0x1008ff11,
        "play" => 0x1008ff14,
        "prev" => 0x1008ff16,
        "next" => 0x1008ff17,
        "briu" => 0x1008ff02,
        "brid" => 0x1008ff03,

        _ => return None,
    })
}
