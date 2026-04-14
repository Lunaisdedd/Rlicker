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
}

// Color
fn bg()      -> Color { Color::from_rgb(0,  0,  0)  }
fn surface() -> Color { Color::from_rgb(20,  20,  20)  }
fn accent()  -> Color { Color::from_rgb(137, 180, 250) }
fn text()    -> Color { Color::from_rgb(255, 255, 255) }
fn muted()   -> Color { Color::from_rgb(108, 112, 134) }
fn green()   -> Color { Color::from_rgb(166, 227, 161) }
fn go_bg()   -> Color { Color::from_rgb(26,  51,  32)  }
fn stop_bg() -> Color { Color::from_rgb(61,  21,  21)  }

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
    });

    // Create a message channel for thread-to-UI communication
    let (tx, rx) = app::channel::<bool>();

    thread::spawn({ let s = state.clone(); move || clicker_loop(s) });
    thread::spawn({ let s = state.clone(); let t = tx.clone(); move || global_listener(s, t) });

    let a = app::App::default().with_scheme(app::Scheme::Gtk);

    let mut win = Window::new(0, 0, 300, 360, "Rlicker");
    win.set_xclass("Rlicker");

    // Spawn window in the exact center of the screen
    let (scr_w, scr_h) = app::screen_size();
    win.set_pos((scr_w as i32 - 300) / 2, (scr_h as i32 - 360) / 2);
    win.set_color(bg());

    //BUTTON
    mkhead("BUTTON", 10, 10);
    let mut rb_l = RadioRoundButton::new(14,  28, 120, 28, "Left");
    let mut rb_r = RadioRoundButton::new(148, 28, 120, 28, "Right");
    for rb in [&mut rb_l, &mut rb_r] {
        rb.set_label_color(text()); rb.set_color(bg()); rb.set_selection_color(accent());
    }
    rb_l.set_value(true);

    //  TIMING
    mkhead("TIMING", 10, 68);
    let (mut iv, mut ivj) = mkrow("Interval (ms)", 1000, 0, 10, 88);
    let (mut hd, mut hdj) = mkrow("Hold (ms)",       50, 0, 10, 122);

    // HOTKEY
    mkhead("HOTKEY", 10, 160);
    let mut frm_hk = Frame::new(14, 178, 158, 28, "None");
    frm_hk.set_frame(FrameType::FlatBox);
    frm_hk.set_color(surface());
    frm_hk.set_label_color(text());
    frm_hk.set_label_size(12);
    frm_hk.set_align(Align::Left | Align::Inside);

    let mut btn_set = mkbtn("Set",   176, 178, 52, 28, Color::from_rgb(69, 71, 90));
    let mut btn_clr = mkbtn("Clear", 232, 178, 58, 28, Color::from_rgb(74, 42, 40));

    let mut chk = CheckButton::new(14, 212, 272, 24, " Hold to click");
    chk.set_label_color(text()); chk.set_color(bg()); chk.set_selection_color(accent());

    //  divider
    let mut div = Frame::new(10, 246, 280, 1, "");
    div.set_frame(FrameType::FlatBox); div.set_color(surface());

    //  status
    let mut lbl_st = Frame::new(14, 254, 272, 24, "● Stopped");
    lbl_st.set_label_color(muted());
    lbl_st.set_label_font(Font::HelveticaBold);
    lbl_st.set_label_size(12);
    lbl_st.set_align(Align::Left | Align::Inside);
    lbl_st.set_frame(FrameType::NoBox);

    //  toggle
    let mut btn_tog = Button::new(10, 286, 280, 54, "START");
    btn_tog.set_color(go_bg());
    btn_tog.set_label_color(text());
    btn_tog.set_label_font(Font::HelveticaBold);
    btn_tog.set_label_size(20);
    btn_tog.set_frame(FrameType::FlatBox);

    win.end();
    win.show();

    // Setup input Callbacks
    { let s = state.clone(); rb_l.set_callback(move |_| s.button_code.store(BTN_LEFT,  Ordering::Relaxed)); }
    { let s = state.clone(); rb_r.set_callback(move |_| s.button_code.store(BTN_RIGHT, Ordering::Relaxed)); }
    { let s = state.clone(); iv .set_callback(move |w| s.interval_ms    .store(w.value().parse().unwrap_or(1).max(1), Ordering::Relaxed)); }
    { let s = state.clone(); ivj.set_callback(move |w| s.interval_jitter.store(w.value().parse().unwrap_or(0),       Ordering::Relaxed)); }
    { let s = state.clone(); hd .set_callback(move |w| s.hold_ms        .store(w.value().parse().unwrap_or(0),       Ordering::Relaxed)); }
    { let s = state.clone(); hdj.set_callback(move |w| s.hold_jitter    .store(w.value().parse().unwrap_or(0),       Ordering::Relaxed)); }

    // Setup Action Callbacks (Sending messages via the tx channel)
    { let s = state.clone(); let t = tx.clone(); btn_set.set_callback(move |_| { s.grabbing.store(true, Ordering::SeqCst); t.send(true); }); }
    { let s = state.clone(); let t = tx.clone(); btn_clr.set_callback(move |_| {
        s.hotkey_code.store(0, Ordering::SeqCst);
        if s.hold_mode.load(Ordering::SeqCst) { set_running(&s, false, &t); } else { t.send(true); }
    }); }
    { let s = state.clone(); chk.set_callback(move |w| s.hold_mode.store(w.value(), Ordering::SeqCst)); }
    { let s = state.clone(); let t = tx.clone(); btn_tog.set_callback(move |_| toggle(&s, &t)); }

    // Main App Loop with Event Receiver
    while a.wait() {
        let mut needs_update = false;

        // Drain the channel queue
        while rx.recv().is_some() {
            needs_update = true;
        }

        if needs_update {
            let code = state.hotkey_code.load(Ordering::SeqCst);
            if state.grabbing.load(Ordering::SeqCst) {
                frm_hk.set_label("Press a key...");
                frm_hk.set_label_color(Color::Yellow);
            } else {
                let lbl = if code == 0 { "None".to_string() } else { format!("Key {code}") };
                frm_hk.set_label(&lbl);
                frm_hk.set_label_color(text());
            }

            if state.running.load(Ordering::SeqCst) {
                lbl_st.set_label("● Running");  lbl_st.set_label_color(green());
                btn_tog.set_label("STOP");      btn_tog.set_color(stop_bg());
            } else {
                lbl_st.set_label("● Stopped");  lbl_st.set_label_color(muted());
                btn_tog.set_label("START");     btn_tog.set_color(go_bg());
            }

            // Redraw the specific widgets that changed
            frm_hk.redraw();
            lbl_st.redraw();
            btn_tog.redraw();
        }
    }
}

//  widget helpers

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

//  state helpers

fn set_running(state: &AppState, on: bool, tx: &app::Sender<bool>) {
    state.running.store(on, Ordering::SeqCst);
    tx.send(true);
}

fn toggle(state: &AppState, tx: &app::Sender<bool>) {
    state.running.fetch_xor(true, Ordering::SeqCst);
    tx.send(true);
}

// timing

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
fn rand_jitter(range: i64) -> i64 {
    if range == 0 { return 0; }
    (rand::random::<u64>() % (range as u64 * 2 + 1)) as i64 - range
}

//  worker threads

fn clicker_loop(state: Arc<AppState>) {
    let mut keys = AttributeSet::<Key>::new();
    keys.insert(Key::BTN_LEFT);
    keys.insert(Key::BTN_RIGHT);
    let mut device = VirtualDeviceBuilder::new().unwrap()
    .name("Rlicker-uinput")
    .with_keys(&keys).unwrap()
    .build().unwrap();

    loop {
        if !state.running.load(Ordering::Relaxed) {
            thread::sleep(Duration::from_millis(5));
            continue;
        }
        let btn             = state.button_code.load(Ordering::Relaxed);
        let interval        = state.interval_ms.load(Ordering::Relaxed) as i64;
        let hold            = state.hold_ms.load(Ordering::Relaxed) as i64;
        let interval_jitter = state.interval_jitter.load(Ordering::Relaxed) as i64;
        let hold_jitter     = state.hold_jitter.load(Ordering::Relaxed) as i64;
        let actual_hold     = (hold + rand_jitter(hold_jitter)).max(0);
        let cycle_start     = Instant::now();

        let _ = device.emit(&[InputEvent::new(EventType::KEY, btn, 1)]);
        if actual_hold > 0 { precise_sleep(Duration::from_millis(actual_hold as u64)); }
        let _ = device.emit(&[InputEvent::new(EventType::KEY, btn, 0)]);

        let elapsed = cycle_start.elapsed();
        let target  = Duration::from_millis((interval + rand_jitter(interval_jitter)).max(1) as u64);
        if target > elapsed { precise_sleep(target - elapsed); }
    }
}

fn global_listener(state: Arc<AppState>, tx: app::Sender<bool>) {
    loop {
        let mut devices: Vec<Device> = evdev::enumerate()
        .map(|(_, d)| d)
        .filter(|d| d.supported_keys().map_or(false, |k| k.contains(Key::KEY_ENTER)))
        .collect();

        if devices.is_empty() { thread::sleep(Duration::from_secs(1)); continue; }

        for _ in 0..200 {
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
                                1 => set_running(&state, true, &tx),
                                0 => set_running(&state, false, &tx),
                                _ => {}
                            }
                        } else if value == 1 {
                            toggle(&state, &tx);
                        }
                    }
                }
            }
            thread::sleep(Duration::from_millis(5));
        }
    }
}
