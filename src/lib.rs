//! Blur+ for Horizon: the mod API wiring of the blur configuration.
//!
//! The 1:1 logic core lives in `runtime.rs`, untouched: the original's
//! config (BlurConfig with its v3 migration), the HSB and #RRGGBB color
//! math, the ten easing curves, the blur-radius, background-alpha and
//! gradient animation handlers, the render-pass flow (onRender /
//! renderBackground / onRenderEnd), the per-screen gates from every
//! upstream mixin, the options coefficient and the debug HUD renderer.
//!
//! The mod API wires the original's config as settings so Horizon's
//! Options > Mods page holds it, same defaults. The blurred draw itself
//! waits on horizon-rust/horizon#19349: the pinned mod API has no
//! screen-render surface a mod can hook to draw its blurred background
//! over the world, and no screen-open surface for the gates to consult.
//! Until one lands the core compiles and its animations run from the
//! settings, and the screens draw unblurred.
use mod_api::{self, ClientHost, RegistrationContext, Registrar, SettingSpec};

mod runtime;

use runtime::{BlurConfig, Easing};

#[derive(Default)]
struct Mod;

/// The mod's id: its settings' namespace.
const MOD_ID: &str = "blur_plus";

#[mod_api::horizon_mod]
impl mod_api::Hooks for Mod {
    /// The original's config fields registered as settings, same defaults.
    fn register(&self, registrar: &mut dyn Registrar, _context: &RegistrationContext) {
        let cfg = BlurConfig::default();
        // Screens: the per-screen gates the original holds as config.
        for (key, label, value) in [
            ("blur_containers", "setting.blur_plus.blur_containers", cfg.blur_containers),
            ("blur_books", "setting.blur_plus.blur_books", cfg.blur_books),
            ("blur_signs", "setting.blur_plus.blur_signs", cfg.blur_signs),
            ("blur_command_blocks", "setting.blur_plus.blur_command_blocks", cfg.blur_command_blocks),
            ("blur_death_screen", "setting.blur_plus.blur_death_screen", cfg.blur_death_screen),
            ("blur_title_screen", "setting.blur_plus.blur_title_screen", cfg.blur_title_screen),
            ("darken_title_screen", "setting.blur_plus.darken_title_screen", cfg.darken_title_screen),
            ("reduce_in_game_blur", "setting.blur_plus.reduce_in_game_blur", cfg.reduce_in_game_blur),
            ("show_screen_id", "setting.blur_plus.show_screen_id", cfg.show_screen_id),
            ("use_gradient", "setting.blur_plus.use_gradient", cfg.use_gradient),
            ("rainbow_mode", "setting.blur_plus.rainbow_mode", cfg.rainbow_mode),
        ] {
            let _ = registrar.register_setting(SettingSpec::toggle(key, label, value));
        }
        // Style: the gradient colors and their alphas.
        let _ = registrar.register_setting(SettingSpec::color(
            "gradient_start",
            "setting.blur_plus.gradient_start",
            color_u32(&cfg.gradient_start),
        ));
        let _ = registrar.register_setting(SettingSpec::range(
            "gradient_start_alpha",
            "setting.blur_plus.gradient_start_alpha",
            0.0,
            255.0,
            1.0,
            cfg.gradient_start_alpha as f64,
        ));
        let _ = registrar.register_setting(SettingSpec::color(
            "gradient_end",
            "setting.blur_plus.gradient_end",
            color_u32(&cfg.gradient_end),
        ));
        let _ = registrar.register_setting(SettingSpec::range(
            "gradient_end_alpha",
            "setting.blur_plus.gradient_end_alpha",
            0.0,
            255.0,
            1.0,
            cfg.gradient_end_alpha as f64,
        ));
        let _ = registrar.register_setting(SettingSpec::range(
            "gradient_rotation",
            "setting.blur_plus.gradient_rotation",
            -180.0,
            180.0,
            1.0,
            cfg.gradient_rotation as f64,
        ));
        // Animations: the fade times and the ten easing curves.
        let _ = registrar.register_setting(SettingSpec::range(
            "fade_time_millis",
            "setting.blur_plus.fade_time_millis",
            0.0,
            5000.0,
            10.0,
            cfg.fade_time_millis as f64,
        ));
        let _ = registrar.register_setting(SettingSpec::range(
            "fade_out_time_millis",
            "setting.blur_plus.fade_out_time_millis",
            0.0,
            5000.0,
            10.0,
            cfg.fade_out_time_millis as f64,
        ));
        let _ = registrar.register_setting(SettingSpec::choice(
            "blur_animation_curve",
            "setting.blur_plus.blur_animation_curve",
            EASING_LABELS,
            easing_index(cfg.blur_animation_curve),
        ));
        let _ = registrar.register_setting(SettingSpec::choice(
            "background_animation_curve",
            "setting.blur_plus.background_animation_curve",
            EASING_LABELS,
            easing_index(cfg.background_animation_curve),
        ));
        // The two screen lists (the original's force-enabled and
        // force-disabled) stay compiled-in: the pinned API has no
        // string-list setting kind to hold them.
    }
}

/// The ten easing curves the original's Easing enum holds, in enum order.
const EASING_LABELS: [&str; 10] = [
    "setting.blur_plus.easing.flat",
    "setting.blur_plus.easing.sine",
    "setting.blur_plus.easing.quad",
    "setting.blur_plus.easing.cubic",
    "setting.blur_plus.easing.quart",
    "setting.blur_plus.easing.quint",
    "setting.blur_plus.easing.expo",
    "setting.blur_plus.easing.circ",
    "setting.blur_plus.easing.back",
    "setting.blur_plus.easing.elastic",
];

/// The choice index of an easing curve, in enum order.
fn easing_index(e: Easing) -> usize {
    match e {
        Easing::Flat => 0,
        Easing::Sine => 1,
        Easing::Quad => 2,
        Easing::Cubic => 3,
        Easing::Quart => 4,
        Easing::Quint => 5,
        Easing::Expo => 6,
        Easing::Circ => 7,
        Easing::Back => 8,
        Easing::Elastic => 9,
    }
}

/// "#RRGGBB" as the color setting's u32.
fn color_u32(hex: &str) -> u32 {
    let b = hex.as_bytes();
    let pair = |i: usize| -> u32 {
        let d = |c: u8| (c as char).to_digit(16).unwrap_or(0) as u32;
        d(b[i]) * 16 + d(b[i + 1])
    };
    (pair(1) << 16) | (pair(3) << 8) | pair(5)
}

/// The settings read back as the runtime's config, through the host's
/// typed readers. Unused until a screen hook lands (#19349), but wired so
/// the runtime runs its animations from the settings the moment one does.
#[allow(dead_code)]
fn read_config(host: &dyn ClientHost) -> BlurConfig {
    let mut cfg = BlurConfig::default();
    let bool_or = |k: &str, d: bool| host.setting_bool(k).unwrap_or(d);
    cfg.blur_containers = bool_or("blur_containers", cfg.blur_containers);
    cfg.blur_books = bool_or("blur_books", cfg.blur_books);
    cfg.blur_signs = bool_or("blur_signs", cfg.blur_signs);
    cfg.blur_command_blocks = bool_or("blur_command_blocks", cfg.blur_command_blocks);
    cfg.blur_death_screen = bool_or("blur_death_screen", cfg.blur_death_screen);
    cfg.blur_title_screen = bool_or("blur_title_screen", cfg.blur_title_screen);
    cfg.darken_title_screen = bool_or("darken_title_screen", cfg.darken_title_screen);
    cfg.reduce_in_game_blur = bool_or("reduce_in_game_blur", cfg.reduce_in_game_blur);
    cfg.show_screen_id = bool_or("show_screen_id", cfg.show_screen_id);
    cfg.use_gradient = bool_or("use_gradient", cfg.use_gradient);
    cfg.rainbow_mode = bool_or("rainbow_mode", cfg.rainbow_mode);
    if let Some(c) = host.setting_color("gradient_start") {
        cfg.gradient_start = format!("#{:06x}", c & 0xffffff);
    }
    if let Some(c) = host.setting_color("gradient_end") {
        cfg.gradient_end = format!("#{:06x}", c & 0xffffff);
    }
    let f64_or = |k: &str, d: f64| host.setting_f64(k).unwrap_or(d);
    cfg.gradient_start_alpha = f64_or("gradient_start_alpha", 75.0) as i32;
    cfg.gradient_end_alpha = f64_or("gradient_end_alpha", 75.0) as i32;
    cfg.gradient_rotation = f64_or("gradient_rotation", 0.0) as i32;
    cfg.fade_time_millis = f64_or("fade_time_millis", 300.0) as i32;
    cfg.fade_out_time_millis = f64_or("fade_out_time_millis", 300.0) as i32;
    if let Some(i) = host.setting_choice("blur_animation_curve") {
        cfg.blur_animation_curve = easing_from_index(i as usize);
    }
    if let Some(i) = host.setting_choice("background_animation_curve") {
        cfg.background_animation_curve = easing_from_index(i as usize);
    }
    cfg
}

/// The easing curve at a choice index, in enum order.
#[allow(dead_code)]
fn easing_from_index(i: usize) -> Easing {
    match i {
        1 => Easing::Sine,
        2 => Easing::Quad,
        3 => Easing::Cubic,
        4 => Easing::Quart,
        5 => Easing::Quint,
        6 => Easing::Expo,
        7 => Easing::Circ,
        8 => Easing::Back,
        9 => Easing::Elastic,
        _ => Easing::Flat,
    }
}
