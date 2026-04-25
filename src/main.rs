use fltk::{
    app,
    button::{Button, CheckButton, RadioRoundButton},
    enums::{Align, Color, Font, FrameType},
    frame::Frame,
    input::IntInput,
    prelude::*,
    window::Window,
};
use evdev::{uinput::VirtualDeviceBuilder, AttributeSet, Device, EventType, InputEvent, Key};
use rand::Rng;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU16, AtomicU32, Ordering};
use std::thread;
use std::time::{Duration, Instant};

const BTN_LEFT:  u16 = 272;
const BTN_RIGHT: u16 = 273;
const KEY_ESC:   u16 = 1;

struct AppState {
    running:         AtomicBool,
    grabbing:        AtomicBool,
    button_code:     AtomicU16,
    interval_ms:     AtomicU32,
    hold_ms:         AtomicU32,
    interval_jitter: AtomicU32,
    hold_jitter:     AtomicU32,
    hotkey_code:     AtomicU16,
    hold_mode:       AtomicBool,
    shutdown:        AtomicBool,
}

// Colors
fn bg()      -> Color { Color::from_rgb(0,   0,   0)   }
fn surface() -> Color { Color::from_rgb(20,  20,  20)  }
fn accent()  -> Color { Color::from_rgb(137, 180, 250) }
fn text()    -> Color { Color::from_rgb(255, 255, 255) }
fn muted()   -> Color { Color::from_rgb(108, 112, 134) }
fn green()   -> Color { Color::from_rgb(166, 227, 161) }
fn go_bg()   -> Color { Color::from_rgb(26,  51,  32)  }
fn stop_bg() -> Color { Color::from_rgb(61,  21,  21)  }

/// Translate an evdev key code to a human-readable name.
fn key_name(code: u16) -> String {
    // Build a temporary AttributeSet containing only this code, then use
    // evdev's Key::from_code for the canonical name.
    match Key::new(code) {
        // Letters
        k if k == Key::KEY_A => "A".into(),
        k if k == Key::KEY_B => "B".into(),
        k if k == Key::KEY_C => "C".into(),
        k if k == Key::KEY_D => "D".into(),
        k if k == Key::KEY_E => "E".into(),
        k if k == Key::KEY_F => "F".into(),
        k if k == Key::KEY_G => "G".into(),
        k if k == Key::KEY_H => "H".into(),
        k if k == Key::KEY_I => "I".into(),
        k if k == Key::KEY_J => "J".into(),
        k if k == Key::KEY_K => "K".into(),
        k if k == Key::KEY_L => "L".into(),
        k if k == Key::KEY_M => "M".into(),
        k if k == Key::KEY_N => "N".into(),
        k if k == Key::KEY_O => "O".into(),
        k if k == Key::KEY_P => "P".into(),
        k if k == Key::KEY_Q => "Q".into(),
        k if k == Key::KEY_R => "R".into(),
        k if k == Key::KEY_S => "S".into(),
        k if k == Key::KEY_T => "T".into(),
        k if k == Key::KEY_U => "U".into(),
        k if k == Key::KEY_V => "V".into(),
        k if k == Key::KEY_W => "W".into(),
        k if k == Key::KEY_X => "X".into(),
        k if k == Key::KEY_Y => "Y".into(),
        k if k == Key::KEY_Z => "Z".into(),
        // Numbers
        k if k == Key::KEY_1 => "1".into(),
        k if k == Key::KEY_2 => "2".into(),
        k if k == Key::KEY_3 => "3".into(),
        k if k == Key::KEY_4 => "4".into(),
        k if k == Key::KEY_5 => "5".into(),
        k if k == Key::KEY_6 => "6".into(),
        k if k == Key::KEY_7 => "7".into(),
        k if k == Key::KEY_8 => "8".into(),
        k if k == Key::KEY_9 => "9".into(),
        k if k == Key::KEY_0 => "0".into(),
        // Function keys
        k if k == Key::KEY_F1  => "F1".into(),
        k if k == Key::KEY_F2  => "F2".into(),
        k if k == Key::KEY_F3  => "F3".into(),
        k if k == Key::KEY_F4  => "F4".into(),
        k if k == Key::KEY_F5  => "F5".into(),
        k if k == Key::KEY_F6  => "F6".into(),
        k if k == Key::KEY_F7  => "F7".into(),
        k if k == Key::KEY_F8  => "F8".into(),
        k if k == Key::KEY_F9  => "F9".into(),
        k if k == Key::KEY_F10 => "F10".into(),
        k if k == Key::KEY_F11 => "F11".into(),
        k if k == Key::KEY_F12 => "F12".into(),
        // Modifiers & common
        k if k == Key::KEY_LEFTSHIFT   => "L.Shift".into(),
        k if k == Key::KEY_RIGHTSHIFT  => "R.Shift".into(),
        k if k == Key::KEY_LEFTCTRL    => "L.Ctrl".into(),
        k if k == Key::KEY_RIGHTCTRL   => "R.Ctrl".into(),
        k if k == Key::KEY_LEFTALT     => "L.Alt".into(),
        k if k == Key::KEY_RIGHTALT    => "R.Alt".into(),
        k if k == Key::KEY_LEFTMETA    => "L.Meta".into(),
        k if k == Key::KEY_RIGHTMETA   => "R.Meta".into(),
        k if k == Key::KEY_TAB         => "Tab".into(),
        k if k == Key::KEY_ENTER       => "Enter".into(),
        k if k == Key::KEY_SPACE       => "Space".into(),
        k if k == Key::KEY_BACKSPACE   => "Backspace".into(),
        k if k == Key::KEY_DELETE      => "Delete".into(),
        k if k == Key::KEY_INSERT      => "Insert".into(),
        k if k == Key::KEY_HOME        => "Home".into(),
        k if k == Key::KEY_END         => "End".into(),
        k if k == Key::KEY_PAGEUP      => "PgUp".into(),
        k if k == Key::KEY_PAGEDOWN    => "PgDn".into(),
        k if k == Key::KEY_UP          => "Up".into(),
        k if k == Key::KEY_DOWN        => "Down".into(),
        k if k == Key::KEY_LEFT        => "Left".into(),
        k if k == Key::KEY_RIGHT       => "Right".into(),
        k if k == Key::KEY_CAPSLOCK    => "CapsLk".into(),
        k if k == Key::KEY_NUMLOCK     => "NumLk".into(),
        k if k == Key::KEY_SCROLLLOCK  => "ScrLk".into(),
        k if k == Key::KEY_PAUSE       => "Pause".into(),
        k if k == Key::KEY_SYSRQ      => "PrtSc".into(),
        // Numpad
        k if k == Key::KEY_KP0        => "Num0".into(),
        k if k == Key::KEY_KP1        => "Num1".into(),
        k if k == Key::KEY_KP2        => "Num2".into(),
        k if k == Key::KEY_KP3        => "Num3".into(),
        k if k == Key::KEY_KP4        => "Num4".into(),
        k if k == Key::KEY_KP5        => "Num5".into(),
        k if k == Key::KEY_KP6        => "Num6".into(),
        k if k == Key::KEY_KP7        => "Num7".into(),
        k if k == Key::KEY_KP8        => "Num8".into(),
        k if k == Key::KEY_KP9        => "Num9".into(),
        k if k == Key::KEY_KPENTER    => "Num Enter".into(),
        k if k == Key::KEY_KPPLUS     => "Num +".into(),
        k if k == Key::KEY_KPMINUS    => "Num -".into(),
        k if k == Key::KEY_KPASTERISK => "Num *".into(),
        k if k == Key::KEY_KPSLASH    => "Num /".into(),
        k if k == Key::KEY_KPDOT      => "Num .".into(),
        _ => format!("Key {code}"),
    }
}

fn main() {
    let state = Arc::new(AppState {
        running:         AtomicBool::new(false),
        grabbing:        AtomicBool::new(false),
        button_code:     AtomicU16::new(BTN_LEFT),
        interval_ms:     AtomicU32::new(1000),
        hold_ms:         AtomicU32::new(50),
        interval_jitter: AtomicU32::new(0),
        hold_jitter:     AtomicU32::new(0),
        hotkey_code:     AtomicU16::new(0),
        hold_mode:       AtomicBool::new(false),
        shutdown:        AtomicBool::new(false),
    });

    let (tx, rx) = app::channel::<bool>();

    // clicker_loop exits on error or shutdown
    thread::spawn({
        let s = state.clone();
        let t = tx.clone();
        move || {
            if let Err(e) = clicker_loop(s.clone()) {
                if !s.shutdown.load(Ordering::SeqCst) {
                    eprintln!("clicker error: {e}");
                    s.shutdown.store(true, Ordering::SeqCst);
                    s.running.store(false, Ordering::SeqCst);
                    t.send(true);
                }
            }
        }
    });

    thread::spawn({
        let s = state.clone();
        let t = tx.clone();
        move || global_listener(s, t)
    });

    let a = app::App::default().with_scheme(app::Scheme::Gtk);

    let mut win = Window::new(0, 0, 300, 360, "Rlicker");
    win.set_xclass("Rlicker");

    let (scr_w, scr_h) = app::screen_size();
    win.set_pos((scr_w as i32 - 300) / 2, (scr_h as i32 - 360) / 2);
    win.set_color(bg());

    // BUTTON
    mkhead("BUTTON", 10, 10);
    let mut rb_l = RadioRoundButton::new(14, 28, 120, 28, "Left");
    let mut rb_r = RadioRoundButton::new(148, 28, 120, 28, "Right");
    for rb in [&mut rb_l, &mut rb_r] {
        rb.set_label_color(text()); rb.set_color(bg()); rb.set_selection_color(accent());
    }
    rb_l.set_value(true);

    // TIMING
    mkhead("TIMING", 10, 68);
    let (mut iv, mut ivj) = mkrow("Interval (ms)", 1000, 0, 10, 88);
    let (mut hd, mut hdj) = mkrow("Hold (ms)", 50, 0, 10, 122);

    // HOTKEY
    mkhead("HOTKEY", 10, 160);
    let mut frm_hk = Frame::new(14, 178, 158, 28, "None");
    frm_hk.set_frame(FrameType::FlatBox);
    frm_hk.set_color(surface());
    frm_hk.set_label_color(text());
    frm_hk.set_label_size(12);
    frm_hk.set_align(Align::Left | Align::Inside);

    let mut btn_set = mkbtn("Set", 176, 178, 52, 28, Color::from_rgb(69, 71, 90));
    let mut btn_clr = mkbtn("Clear", 232, 178, 58, 28, Color::from_rgb(74, 42, 40));

    let mut chk = CheckButton::new(14, 212, 272, 24, " Hold to click");
    chk.set_label_color(text()); chk.set_color(bg()); chk.set_selection_color(accent());

    let mut div = Frame::new(10, 246, 280, 1, "");
    div.set_frame(FrameType::FlatBox); div.set_color(surface());

    let mut lbl_st = Frame::new(14, 254, 272, 24, "Stopped");
    lbl_st.set_label_color(muted());
    lbl_st.set_label_font(Font::HelveticaBold);
    lbl_st.set_label_size(12);
    lbl_st.set_align(Align::Left | Align::Inside);
    lbl_st.set_frame(FrameType::NoBox);

    let mut btn_tog = Button::new(10, 286, 280, 54, "START");
    btn_tog.set_color(go_bg());
    btn_tog.set_label_color(text());
    btn_tog.set_label_font(Font::HelveticaBold);
    btn_tog.set_label_size(20);
    btn_tog.set_frame(FrameType::FlatBox);

    win.end();
    win.show();

    // Input callbacks
    { let s = state.clone(); rb_l.set_callback(move |_| s.button_code.store(BTN_LEFT,  Ordering::Relaxed)); }
    { let s = state.clone(); rb_r.set_callback(move |_| s.button_code.store(BTN_RIGHT, Ordering::Relaxed)); }
    { let s = state.clone(); iv.set_callback(move  |w| s.interval_ms.store(w.value().parse().unwrap_or(1).max(1), Ordering::Relaxed)); }
    { let s = state.clone(); ivj.set_callback(move |w| s.interval_jitter.store(w.value().parse().unwrap_or(0), Ordering::Relaxed)); }
    { let s = state.clone(); hd.set_callback(move  |w| s.hold_ms.store(w.value().parse().unwrap_or(0), Ordering::Relaxed)); }
    { let s = state.clone(); hdj.set_callback(move |w| s.hold_jitter.store(w.value().parse().unwrap_or(0), Ordering::Relaxed)); }

    // Action callbacks
    { let s = state.clone(); let t = tx.clone();
      btn_set.set_callback(move |_| transition(&s, &t, Msg::StartGrab)); }
    { let s = state.clone(); let t = tx.clone();
      btn_clr.set_callback(move |_| transition(&s, &t, Msg::ClearHotkey)); }
    { let s = state.clone(); chk.set_callback(move |w| s.hold_mode.store(w.value(), Ordering::SeqCst)); }
    { let s = state.clone(); let t = tx.clone();
      btn_tog.set_callback(move |_| transition(&s, &t, Msg::Toggle)); }

    // Clean shutdown on window close
    { let s = state.clone(); let t = tx.clone();
      win.set_callback(move |w| {
          s.shutdown.store(true, Ordering::SeqCst);
          s.running.store(false, Ordering::SeqCst);
          t.send(true);
          w.hide();
      });
    }

    while a.wait() {
        if rx.recv().is_some() {
            // Shutdown: drain and quit
            if state.shutdown.load(Ordering::SeqCst) {
                a.quit();
                break;
            }

            let code = state.hotkey_code.load(Ordering::SeqCst);
            if state.grabbing.load(Ordering::SeqCst) {
                frm_hk.set_label("Press a key...");
                frm_hk.set_label_color(Color::Yellow);
            } else {
                let lbl = if code == 0 { "None".to_owned() } else { key_name(code) };
                frm_hk.set_label(&lbl);
                frm_hk.set_label_color(text());
            }

            if state.running.load(Ordering::SeqCst) {
                lbl_st.set_label("Running"); lbl_st.set_label_color(green());
                btn_tog.set_label("STOP");   btn_tog.set_color(stop_bg());
            } else {
                lbl_st.set_label("Stopped"); lbl_st.set_label_color(muted());
                btn_tog.set_label("START");  btn_tog.set_color(go_bg());
            }
            win.redraw();
        }
    }
}

//Centralized state transition

enum Msg { Toggle, StartGrab, ClearHotkey }

fn transition(state: &AppState, tx: &app::Sender<bool>, msg: Msg) {
    match msg {
        Msg::Toggle => { state.running.fetch_xor(true, Ordering::SeqCst); }
        Msg::StartGrab => { state.grabbing.store(true, Ordering::SeqCst); }
        Msg::ClearHotkey => {
            state.hotkey_code.store(0, Ordering::SeqCst);
            if state.hold_mode.load(Ordering::SeqCst) {
                state.running.store(false, Ordering::SeqCst);
            }
        }
    }
    tx.send(true);
}

// Widget helpers

fn mkhead(label: &str, x: i32, y: i32) {
    let mut f = Frame::new(x, y, 280, 14, label);
    f.set_label_color(accent());
    f.set_label_font(Font::HelveticaBold);
    f.set_label_size(10);
    f.set_align(Align::Left | Align::Inside);
    f.set_frame(FrameType::NoBox);
}

fn mkbtn(label: &str, x: i32, y: i32, w: i32, h: i32, bg: Color) -> Button {
    let mut b = Button::new(x, y, w, h, label);
    b.set_color(bg); b.set_label_color(text());
    b.set_label_size(12); b.set_frame(FrameType::FlatBox);
    b
}

fn mkrow(label: &str, val: i32, jval: i32, x: i32, y: i32) -> (IntInput, IntInput) {
    let mut lf = Frame::new(x, y, 90, 28, label);
    lf.set_label_color(text()); lf.set_label_size(12);
    lf.set_align(Align::Left | Align::Inside); lf.set_frame(FrameType::NoBox);

    let mut i1 = IntInput::new(x + 94, y, 72, 28, "");
    i1.set_color(surface()); i1.set_text_color(text()); i1.set_value(&val.to_string());

    let mut jf = Frame::new(x + 170, y, 28, 28, "+/-");
    jf.set_label_color(muted()); jf.set_label_size(11); jf.set_frame(FrameType::NoBox);

    let mut i2 = IntInput::new(x + 202, y, 72, 28, "");
    i2.set_color(surface()); i2.set_text_color(text()); i2.set_value(&jval.to_string());

    (i1, i2)
}

// Timing helpers

#[inline]
fn precise_sleep(dur: Duration) {
    if dur.is_zero() { return; }
    let deadline = Instant::now() + dur;
    if dur > Duration::from_micros(1_500) {
        thread::sleep(dur - Duration::from_micros(500));
    }
    while Instant::now() < deadline { std::hint::spin_loop(); }
}

#[inline]
fn rand_jitter(rng: &mut impl Rng, range: i64) -> i64 {
    if range == 0 { return 0; }
    rng.gen_range(-range..=range)
}

// Worker threads

fn clicker_loop(state: Arc<AppState>) -> Result<(), Box<dyn std::error::Error>> {
    let mut keys = AttributeSet::<Key>::new();
    keys.insert(Key::BTN_LEFT);
    keys.insert(Key::BTN_RIGHT);

    let mut device = VirtualDeviceBuilder::new()
        .map_err(|e| format!("VirtualDeviceBuilder::new failed: {e}"))?
        .name("Rlicker-uinput")
        .with_keys(&keys)
        .map_err(|e| format!("with_keys failed: {e}"))?
        .build()
        .map_err(|e| format!("build failed (is uinput loaded?): {e}"))?;

    let mut rng = rand::thread_rng();

    loop {
        if state.shutdown.load(Ordering::Relaxed) { break; }

        if !state.running.load(Ordering::Relaxed) {
            thread::sleep(Duration::from_millis(5));
            continue;
        }

        let btn             = state.button_code.load(Ordering::Relaxed);
        let interval        = state.interval_ms.load(Ordering::Relaxed) as i64;
        let hold            = state.hold_ms.load(Ordering::Relaxed) as i64;
        let interval_jitter = state.interval_jitter.load(Ordering::Relaxed) as i64;
        let hold_jitter     = state.hold_jitter.load(Ordering::Relaxed) as i64;
        let actual_hold     = (hold + rand_jitter(&mut rng, hold_jitter)).max(0);
        let cycle_start     = Instant::now();

        let _ = device.emit(&[InputEvent::new(EventType::KEY, btn, 1)]);
        if actual_hold > 0 { precise_sleep(Duration::from_millis(actual_hold as u64)); }
        let _ = device.emit(&[InputEvent::new(EventType::KEY, btn, 0)]);

        let elapsed = cycle_start.elapsed();
        let target  = Duration::from_millis(
            (interval + rand_jitter(&mut rng, interval_jitter)).max(1) as u64
        );
        if target > elapsed { precise_sleep(target - elapsed); }
    }

    Ok(())
}

fn global_listener(state: Arc<AppState>, tx: app::Sender<bool>) {
    let mut devices: Vec<Device> = Vec::with_capacity(8);
    loop {
        if state.shutdown.load(Ordering::Relaxed) { break; }

        devices.clear();
        devices.extend(
            evdev::enumerate()
                .map(|(_, d)| d)
                .filter(|d| d.supported_keys().map_or(false, |k| k.contains(Key::KEY_ENTER)))
        );

        if devices.is_empty() { thread::sleep(Duration::from_secs(1)); continue; }

        for _ in 0..200 {
            if state.shutdown.load(Ordering::Relaxed) { return; }

            for device in &mut devices {
                while let Ok(events) = device.fetch_events() {
                    for ev in events {
                        if ev.event_type() != EventType::KEY { continue; }
                        let code  = ev.code();
                        let value = ev.value();

                        if state.grabbing.load(Ordering::SeqCst) {
                            if value != 1 { continue; }
                            if code == KEY_ESC {
                                state.grabbing.store(false, Ordering::SeqCst);
                            } else {
                                state.hotkey_code.store(code, Ordering::SeqCst);
                                state.grabbing.store(false, Ordering::SeqCst);
                                thread::sleep(Duration::from_millis(200));
                            }
                            tx.send(true);
                            continue;
                        }

                        let target = state.hotkey_code.load(Ordering::SeqCst);
                        if target == 0 || code != target { continue; }

                        if state.hold_mode.load(Ordering::SeqCst) {
                            match value {
                                1 => transition(&state, &tx, Msg::Toggle), // sets running=true via xor (was false)
                                0 => { state.running.store(false, Ordering::SeqCst); tx.send(true); }
                                _ => {}
                            }
                        } else if value == 1 {
                            transition(&state, &tx, Msg::Toggle);
                        }
                    }
                }
            }
            thread::sleep(Duration::from_millis(5));
        }
    }
}
