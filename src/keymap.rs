use x11::keysym;

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
    keys
}

pub fn parse_string(s: &str) -> Option<u64> {
    let s = s.to_lowercase();
    Some(match s.as_str() {
        // Letters
        "a" => keysym::XK_a,
        "b" => keysym::XK_b,
        "c" => keysym::XK_c,
        "d" => keysym::XK_d,
        "e" => keysym::XK_e,
        "f" => keysym::XK_f,
        "g" => keysym::XK_g,
        "h" => keysym::XK_h,
        "i" => keysym::XK_i,
        "j" => keysym::XK_j,
        "k" => keysym::XK_k,
        "l" => keysym::XK_l,
        "m" => keysym::XK_m,
        "n" => keysym::XK_n,
        "o" => keysym::XK_o,
        "p" => keysym::XK_p,
        "q" => keysym::XK_q,
        "r" => keysym::XK_r,
        "s" => keysym::XK_s,
        "t" => keysym::XK_t,
        "u" => keysym::XK_u,
        "v" => keysym::XK_v,
        "w" => keysym::XK_w,
        "x" => keysym::XK_x,
        "y" => keysym::XK_y,
        "z" => keysym::XK_z,

        // Numbers
        "0" => keysym::XK_0,
        "1" => keysym::XK_1,
        "2" => keysym::XK_2,
        "3" => keysym::XK_3,
        "4" => keysym::XK_4,
        "5" => keysym::XK_5,
        "6" => keysym::XK_6,
        "7" => keysym::XK_7,
        "8" => keysym::XK_8,
        "9" => keysym::XK_9,

        // Function Keys (lowercase f)
        "f1" => keysym::XK_F1,
        "f2" => keysym::XK_F2,
        "f3" => keysym::XK_F3,
        "f4" => keysym::XK_F4,
        "f5" => keysym::XK_F5,
        "f6" => keysym::XK_F6,
        "f7" => keysym::XK_F7,
        "f8" => keysym::XK_F8,
        "f9" => keysym::XK_F9,
        "f10" => keysym::XK_F10,
        "f11" => keysym::XK_F11,
        "f12" => keysym::XK_F12,

        // Special Keys
        "spc" | "space" => keysym::XK_space,
        "ret" | "enter" => keysym::XK_Return,
        "esc" => keysym::XK_Escape,
        "tab" => keysym::XK_Tab,
        "bsp" | "backspace" => keysym::XK_BackSpace,
        "del" => keysym::XK_Delete,
        "ins" => keysym::XK_Insert,
        "home" => keysym::XK_Home,
        "end" => keysym::XK_End,
        "pgup" => keysym::XK_Page_Up,
        "pgdn" => keysym::XK_Page_Down,

        // Arrow Keys
        "up" => keysym::XK_Up,
        "down" => keysym::XK_Down,
        "left" => keysym::XK_Left,
        "right" => keysym::XK_Right,

        // Punctuation (unshifted versions)
        "-" => keysym::XK_minus,
        "=" => keysym::XK_equal,
        "[" => keysym::XK_bracketleft,
        "]" => keysym::XK_bracketright,
        "\\" => keysym::XK_backslash,
        ";" => keysym::XK_semicolon,
        "'" => keysym::XK_apostrophe,
        "," => keysym::XK_comma,
        "." => keysym::XK_period,
        "/" => keysym::XK_slash,
        "`" => keysym::XK_grave,

        // Keypad
        "kp0" => keysym::XK_KP_0,
        "kp1" => keysym::XK_KP_1,
        "kp2" => keysym::XK_KP_2,
        "kp3" => keysym::XK_KP_3,
        "kp4" => keysym::XK_KP_4,
        "kp5" => keysym::XK_KP_5,
        "kp6" => keysym::XK_KP_6,
        "kp7" => keysym::XK_KP_7,
        "kp8" => keysym::XK_KP_8,
        "kp9" => keysym::XK_KP_9,
        "kp+" => keysym::XK_KP_Add,
        "kp-" => keysym::XK_KP_Subtract,
        "kp*" => keysym::XK_KP_Multiply,
        "kp/" => keysym::XK_KP_Divide,
        "kp." => keysym::XK_KP_Decimal,
        "kpret" => keysym::XK_KP_Enter,

        // Locks
        "caps" => keysym::XK_Caps_Lock,
        "num" => keysym::XK_Num_Lock,

        // Multimedia
        "mute" => keysym::XF86XK_AudioMute,
        "volu" => keysym::XF86XK_AudioRaiseVolume,
        "vold" => keysym::XF86XK_AudioLowerVolume,
        "play" => keysym::XF86XK_AudioPlay,
        "prev" => keysym::XF86XK_AudioPrev,
        "next" => keysym::XF86XK_AudioNext,
        "briu" => keysym::XF86XK_MonBrightnessUp,
        "brid" => keysym::XF86XK_MonBrightnessDown,

        _ => return None,
    } as u64)
}
