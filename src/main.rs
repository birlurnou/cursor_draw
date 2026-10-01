use std::collections::HashMap;
use std::fs;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

// ==================== КОНФИГ ====================

#[derive(Debug, Clone)]
struct Config {
    area_x1: i32,
    area_y1: i32,
    area_x2: i32,
    area_y2: i32,

    char_w: i32,
    char_h: i32,
    glyph_w: i32,
    line_gap: i32,

    px_delay_ms: u64,
    hover_delay_ms: u64,
    after_down_ms: u64,
    before_up_ms: u64,
    stroke_gap_ms: u64,

    poll_interval_ms: u64,

    use_ctrl: bool,

    all_area_keys: Vec<char>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            area_x1: 400,
            area_y1: 400,
            area_x2: 1400,
            area_y2: 1000,

            char_w: 10,
            char_h: 15,
            glyph_w: 7,
            line_gap: 5,

            px_delay_ms: 1,
            hover_delay_ms: 10,
            after_down_ms: 10,
            before_up_ms: 10,
            stroke_gap_ms: 10,

            poll_interval_ms: 20,

            use_ctrl: false,

            all_area_keys: Vec::new(),
        }
    }
}

impl Config {
    fn load(path: &str) -> Self {
        let mut cfg = Config::default();

        let text = match fs::read_to_string(path) {
            Ok(t) => t,
            Err(_) => {
                eprintln!("[CFG] {} не найден, использую значения по умолчанию", path);
                return cfg;
            }
        };

        for raw in text.lines() {
            let line = raw.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let Some((k, v)) = line.split_once('=') else {
                eprintln!("[CFG] пропускаю строку без '=': {}", line);
                continue;
            };
            let key = k.trim().to_ascii_lowercase();
            let val = v.trim();

            let parse_i32 = |v: &str| -> Option<i32> { v.parse::<i32>().ok() };
            let parse_u64 = |v: &str| -> Option<u64> { v.parse::<u64>().ok() };

            match key.as_str() {
                "area_x1"         => if let Some(x) = parse_i32(val) { cfg.area_x1 = x } else { eprintln!("[CFG] плохое значение area_x1 = {}", val) },
                "area_y1"         => if let Some(x) = parse_i32(val) { cfg.area_y1 = x } else { eprintln!("[CFG] плохое значение area_y1 = {}", val) },
                "area_x2"         => if let Some(x) = parse_i32(val) { cfg.area_x2 = x } else { eprintln!("[CFG] плохое значение area_x2 = {}", val) },
                "area_y2"         => if let Some(x) = parse_i32(val) { cfg.area_y2 = x } else { eprintln!("[CFG] плохое значение area_y2 = {}", val) },
                "char_w"          => if let Some(x) = parse_i32(val) { cfg.char_w = x } else { eprintln!("[CFG] плохое значение char_w = {}", val) },
                "char_h"          => if let Some(x) = parse_i32(val) { cfg.char_h = x } else { eprintln!("[CFG] плохое значение char_h = {}", val) },
                "glyph_w"         => if let Some(x) = parse_i32(val) { cfg.glyph_w = x } else { eprintln!("[CFG] плохое значение glyph_w = {}", val) },
                "line_gap"        => if let Some(x) = parse_i32(val) { cfg.line_gap = x } else { eprintln!("[CFG] плохое значение line_gap = {}", val) },
                "px_delay_ms"     => if let Some(x) = parse_u64(val) { cfg.px_delay_ms = x } else { eprintln!("[CFG] плохое значение px_delay_ms = {}", val) },
                "hover_delay_ms"  => if let Some(x) = parse_u64(val) { cfg.hover_delay_ms = x } else { eprintln!("[CFG] плохое значение hover_delay_ms = {}", val) },
                "after_down_ms"   => if let Some(x) = parse_u64(val) { cfg.after_down_ms = x } else { eprintln!("[CFG] плохое значение after_down_ms = {}", val) },
                "before_up_ms"    => if let Some(x) = parse_u64(val) { cfg.before_up_ms = x } else { eprintln!("[CFG] плохое значение before_up_ms = {}", val) },
                "stroke_gap_ms"   => if let Some(x) = parse_u64(val) { cfg.stroke_gap_ms = x } else { eprintln!("[CFG] плохое значение stroke_gap_ms = {}", val) },
                "poll_interval_ms"=> if let Some(x) = parse_u64(val) { cfg.poll_interval_ms = x } else { eprintln!("[CFG] плохое значение poll_interval_ms = {}", val) },
                "use_ctrl"        => {
                    match parse_bool(val) {
                        Some(b) => cfg.use_ctrl = b,
                        None => eprintln!("[CFG] плохое значение use_ctrl = {} (ожидается 1/0, true/false, yes/no, on/off)", val),
                    }
                }
                "all_area_keys" | "all_area" => {
                    cfg.all_area_keys.clear();
                    for tok in val.split(|c: char| c == ',' || c == ';' || c.is_whitespace()) {
                        let t = tok.trim();
                        if let Some(c) = t.chars().next() {
                            cfg.all_area_keys.push(normalize(c));
                        }
                    }
                }
                other => eprintln!("[CFG] неизвестный ключ: {}", other),
            }
        }

        if cfg.char_w <= 0 { eprintln!("[CFG] char_w <= 0, ставлю 10"); cfg.char_w = 10; }
        if cfg.char_h <= 0 { eprintln!("[CFG] char_h <= 0, ставлю 15"); cfg.char_h = 15; }
        if cfg.glyph_w <= 0 { eprintln!("[CFG] glyph_w <= 0, ставлю 7"); cfg.glyph_w = 7; }
        if cfg.line_gap < 0 { eprintln!("[CFG] line_gap < 0, ставлю 0"); cfg.line_gap = 0; }
        if cfg.glyph_w > cfg.char_w {
            eprintln!("[CFG] glyph_w > char_w — буквы будут налезать друг на друга");
        }
        if cfg.area_x2 <= cfg.area_x1 {
            eprintln!("[CFG] area_x2 <= area_x1, ставлю area_x2 = area_x1 + 10*char_w");
            cfg.area_x2 = cfg.area_x1 + 10 * cfg.char_w;
        }
        if cfg.area_y2 <= cfg.area_y1 {
            eprintln!("[CFG] area_y2 <= area_y1, ставлю area_y2 = area_y1 + 10*(char_h+line_gap)");
            cfg.area_y2 = cfg.area_y1 + 10 * (cfg.char_h + cfg.line_gap);
        }
        if cfg.px_delay_ms == 0 {
            eprintln!("[CFG] px_delay_ms = 0 — приложение-цель может не успевать за курсором");
        }

        cfg
    }

    fn chars_per_line(&self) -> i32 {
        let w = self.area_x2 - self.area_x1;
        (w / self.char_w).max(1)
    }

    fn lines_count(&self) -> i32 {
        let h = self.area_y2 - self.area_y1;
        let step = self.char_h + self.line_gap;
        if step <= 0 { return 1; }
        (h / step).max(1)
    }

    fn is_all_area_key(&self, ch: char) -> bool {
        let key = normalize(ch);
        self.all_area_keys.iter().any(|c| *c == key)
    }
}

fn parse_bool(s: &str) -> Option<bool> {
    match s.trim().to_ascii_lowercase().as_str() {
        "1" | "true"  | "yes" | "on"  | "y" => Some(true),
        "0" | "false" | "no"  | "off" | "n" => Some(false),
        _ => None,
    }
}

// ==================== WINDOWS API ====================
#[cfg(windows)]
mod win {
    use winapi::shared::minwindef::DWORD;
    use winapi::shared::windef::POINT;
    use winapi::um::winuser::{
        GetAsyncKeyState, GetCursorPos, GetSystemMetrics, SendInput,
        INPUT, INPUT_KEYBOARD, INPUT_MOUSE,
        KEYBDINPUT, KEYEVENTF_KEYUP,
        MOUSEEVENTF_ABSOLUTE, MOUSEEVENTF_LEFTDOWN,
        MOUSEEVENTF_LEFTUP, MOUSEEVENTF_MOVE, MOUSEINPUT,
        SM_CXSCREEN, SM_CYSCREEN,
        VK_BACK, VK_CONTROL, VK_ESCAPE, VK_RETURN, VK_SHIFT, VK_TAB,
    };

    fn normalize(x: i32, y: i32) -> (i32, i32) {
        unsafe {
            let sw = GetSystemMetrics(SM_CXSCREEN).max(1);
            let sh = GetSystemMetrics(SM_CYSCREEN).max(1);
            let nx = ((x as i64 * 65535) / (sw as i64 - 1).max(1)) as i32;
            let ny = ((y as i64 * 65535) / (sh as i64 - 1).max(1)) as i32;
            (nx, ny)
        }
    }

    fn send_mouse(flags: DWORD, x: i32, y: i32) -> u32 {
        unsafe {
            let (nx, ny) = normalize(x, y);
            let mut input: INPUT = std::mem::zeroed();
            input.type_ = INPUT_MOUSE;
            let mi = &mut *(&mut input.u as *mut _ as *mut MOUSEINPUT);
            mi.dx = nx;
            mi.dy = ny;
            mi.mouseData = 0;
            mi.dwFlags = flags | MOUSEEVENTF_ABSOLUTE;
            mi.time = 0;
            mi.dwExtraInfo = 0;
            SendInput(1, &mut input, std::mem::size_of::<INPUT>() as i32)
        }
    }

    pub fn move_input(x: i32, y: i32) {
        let sent = send_mouse(MOUSEEVENTF_MOVE, x, y);
        if sent != 1 {
            eprintln!("[WARN] SendInput MOVE вернул {}", sent);
        }
    }

    pub fn left_down_at(x: i32, y: i32) {
        let sent = send_mouse(MOUSEEVENTF_LEFTDOWN, x, y);
        if sent != 1 {
            eprintln!("[WARN] SendInput LEFT_DOWN вернул {}", sent);
        }
    }

    pub fn left_up_at(x: i32, y: i32) {
        let sent = send_mouse(MOUSEEVENTF_LEFTUP, x, y);
        if sent != 1 {
            eprintln!("[WARN] SendInput LEFT_UP вернул {}", sent);
        }
    }

    fn send_key(vk: u16, up: bool) -> u32 {
        unsafe {
            let mut input: INPUT = std::mem::zeroed();
            input.type_ = INPUT_KEYBOARD;
            let ki = &mut *(&mut input.u as *mut _ as *mut KEYBDINPUT);
            ki.wVk = vk;
            ki.wScan = 0;
            ki.dwFlags = if up { KEYEVENTF_KEYUP } else { 0 };
            ki.time = 0;
            ki.dwExtraInfo = 0;
            SendInput(1, &mut input, std::mem::size_of::<INPUT>() as i32)
        }
    }

    pub fn ctrl_down() {
        let sent = send_key(VK_CONTROL as u16, false);
        if sent != 1 { eprintln!("[WARN] SendInput CTRL_DOWN вернул {}", sent); }
    }
    pub fn ctrl_release_safe() {
        for _ in 0..2 {
            let _ = send_key(0xA2, true); // VK_LCONTROL up
            let _ = send_key(0xA3, true); // VK_RCONTROL up
            let _ = send_key(VK_CONTROL as u16, true);
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
    }

    pub fn key_was_pressed(vk: i32) -> bool {
        unsafe { (GetAsyncKeyState(vk) as u16 & 0x0001) != 0 }
    }
    pub fn key_down(vk: i32) -> bool {
        unsafe { (GetAsyncKeyState(vk) as u16 & 0x8000) != 0 }
    }

    pub fn enter_was_pressed()     -> bool { key_was_pressed(VK_RETURN as i32) }
    pub fn escape_was_pressed()    -> bool { key_was_pressed(VK_ESCAPE as i32) }
    pub fn backspace_was_pressed() -> bool { key_was_pressed(VK_BACK as i32) }
    pub fn shift_was_pressed()     -> bool {
        // Фронт Shift (левого, правого или generic VK_SHIFT).
        key_was_pressed(VK_SHIFT as i32) || key_was_pressed(0xA0) || key_was_pressed(0xA1)
    }
    /// Текущее состояние Shift — зажат ли он ПРЯМО СЕЙЧАС.
    /// Нужен для детекта «Shift+Enter» (Shift нажат раньше Enter).
    pub fn shift_is_down() -> bool {
        key_down(VK_SHIFT as i32) || key_down(0xA0) || key_down(0xA1)
    }
    pub fn tab_was_pressed()       -> bool { key_was_pressed(VK_TAB as i32) }

    pub fn get_cursor_pos() -> (i32, i32) {
        unsafe {
            let mut p = POINT { x: 0, y: 0 };
            GetCursorPos(&mut p);
            (p.x, p.y)
        }
    }

    pub fn boost_timer() {
        use winapi::um::timeapi::timeBeginPeriod;
        unsafe { timeBeginPeriod(1); }
    }

    fn interesting_vks() -> &'static [i32] {
        &[
            0x30,0x31,0x32,0x33,0x34,0x35,0x36,0x37,0x38,0x39,
            0x41,0x42,0x43,0x44,0x45,0x46,0x47,0x48,0x49,0x4A,
            0x4B,0x4C,0x4D,0x4E,0x4F,0x50,0x51,0x52,0x53,0x54,
            0x55,0x56,0x57,0x58,0x59,0x5A,
            0x20,
            0xBA, 0xBC, 0xBE, 0xDE, 0xDB, 0xDD, 0xC0,
        ]
    }

    pub fn layout_is_russian() -> bool {
        use winapi::um::winuser::{
            GetKeyboardLayout, GetWindowThreadProcessId, GetForegroundWindow,
        };
        unsafe {
            let hwnd = GetForegroundWindow();
            let tid = GetWindowThreadProcessId(hwnd, std::ptr::null_mut());
            let hkl = GetKeyboardLayout(tid);
            (hkl as usize & 0xFFFF) == 0x0419
        }
    }

    pub fn poll_new_keys(prev: &mut [bool; 256]) -> Vec<i32> {
        let mut out = Vec::new();
        for &vk in interesting_vks() {
            let now = key_down(vk);
            if now && !prev[vk as usize] {
                out.push(vk);
            }
            prev[vk as usize] = now;
        }
        out
    }

    pub fn vk_to_char(vk: i32, russian: bool) -> Option<char> {
        if !russian {
            return match vk {
                0x30..=0x39 => char::from_u32(b'0' as u32 + (vk as u32 - 0x30)),
                0x41..=0x5A => char::from_u32(b'a' as u32 + (vk as u32 - 0x41)),
                0x20 => Some(' '),
                _ => None,
            };
        }
        let c = match vk {
            0x51 => 'й', 0x57 => 'ц', 0x45 => 'у', 0x52 => 'к', 0x54 => 'е',
            0x59 => 'н', 0x55 => 'г', 0x49 => 'ш', 0x4F => 'щ', 0x50 => 'з',
            0xDB => 'х', 0xDD => 'ъ', 0x41 => 'ф', 0x53 => 'ы', 0x44 => 'в',
            0x46 => 'а', 0x47 => 'п', 0x48 => 'р', 0x4A => 'о', 0x4B => 'л',
            0x4C => 'д', 0xBA => 'ж', 0xDE => 'э', 0x5A => 'я', 0x58 => 'ч',
            0x43 => 'с', 0x56 => 'м', 0x42 => 'и', 0x4E => 'т', 0x4D => 'ь',
            0xBC => 'б', 0xBE => 'ю', 0xC0 => 'ё',
            0x30..=0x39 => char::from_u32(b'0' as u32 + (vk as u32 - 0x30))?,
            0x20 => ' ',
            _ => return None,
        };
        Some(c)
    }
}

#[cfg(not(windows))]
mod win {
    pub fn move_input(_x: i32, _y: i32) {}
    pub fn left_down_at(_x: i32, _y: i32) {}
    pub fn left_up_at(_x: i32, _y: i32) {}
    pub fn ctrl_down() {}
    pub fn ctrl_release_safe() {}
    pub fn key_was_pressed(_vk: i32) -> bool { false }
    pub fn key_down(_vk: i32) -> bool { false }
    pub fn enter_was_pressed() -> bool { false }
    pub fn escape_was_pressed() -> bool { false }
    pub fn backspace_was_pressed() -> bool { false }
    pub fn shift_was_pressed() -> bool { false }
    pub fn shift_is_down() -> bool { false }
    pub fn tab_was_pressed() -> bool { false }
    pub fn get_cursor_pos() -> (i32, i32) { (0, 0) }
    pub fn boost_timer() {}
    pub fn layout_is_russian() -> bool { false }
    pub fn poll_new_keys(_prev: &mut [bool; 256]) -> Vec<i32> { Vec::new() }
    pub fn vk_to_char(_vk: i32, _russian: bool) -> Option<char> { None }
}
// =====================================================

type Pt = (i32, i32);
type Glyph = Vec<Vec<Pt>>;

// ==================== НОРМАЛИЗАЦИЯ РЕГИСТРА ====================

fn normalize(ch: char) -> char {
    match ch {
        'а'..='я' | 'ё' => ch.to_uppercase().next().unwrap_or(ch),
        'a'..='z' => ch.to_ascii_uppercase(),
        _ => ch,
    }
}

// ==================== ЗАГРУЗКА ГЛИФОВ ====================

fn load_glyphs(path: &str) -> HashMap<char, Glyph> {
    let text = fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("не могу прочитать {}: {}", path, e));

    let mut map: HashMap<char, Glyph> = HashMap::new();
    let mut cur: Option<char> = None;
    let mut strokes: Vec<Vec<Pt>> = Vec::new();

    fn flush(
        map: &mut HashMap<char, Glyph>,
        cur: &mut Option<char>,
        strokes: &mut Vec<Vec<Pt>>,
    ) {
        if let Some(ch) = cur.take() {
            map.insert(ch, std::mem::take(strokes));
        }
    }

    for raw in text.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if line.starts_with('[') && line.ends_with(']') {
            flush(&mut map, &mut cur, &mut strokes);
            let inside = &line[1..line.len() - 1];
            cur = Some(inside.chars().next().unwrap_or(' '));
        } else {
            let mut stroke: Vec<Pt> = Vec::new();
            for pair in line.split_whitespace() {
                let mut it = pair.split(',');
                let x_str = it.next().unwrap_or_else(|| panic!("плохая точка: {}", pair));
                let y_str = it.next().unwrap_or_else(|| panic!("плохая точка: {}", pair));
                let x: i32 = x_str.trim().parse()
                    .unwrap_or_else(|_| panic!("не число: {}", x_str));
                let y: i32 = y_str.trim().parse()
                    .unwrap_or_else(|_| panic!("не число: {}", y_str));
                stroke.push((x, y));
            }
            strokes.push(stroke);
        }
    }
    flush(&mut map, &mut cur, &mut strokes);
    map
}

// ==================== ГЕОМЕТРИЯ ====================

fn to_screen_scaled(base_x: i32, base_y: i32, sx: i32, sy: i32, p: Pt) -> (i32, i32) {
    (
        base_x + (p.0 * sx) / 1000,
        base_y + (p.1 * sy) / 1000,
    )
}

fn bresenham(from: (i32, i32), to: (i32, i32)) -> Vec<(i32, i32)> {
    let (x0, y0) = from;
    let (x1, y1) = to;
    let dx = (x1 - x0).abs();
    let dy = -(y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;
    let (mut x, mut y) = (x0, y0);

    let mut out = Vec::new();
    loop {
        if x == x1 && y == y1 { break; }
        let e2 = 2 * err;
        if e2 >= dy { err += dy; x += sx; }
        if e2 <= dx { err += dx; y += sy; }
        out.push((x, y));
    }
    out
}

// ==================== РИСОВАНИЕ ====================

fn draw_stroke_scaled(
    cfg: &Config,
    base_x: i32,
    base_y: i32,
    sx: i32,
    sy: i32,
    path: &[Pt],
) {
    if path.is_empty() { return; }

    let start = to_screen_scaled(base_x, base_y, sx, sy, path[0]);

    win::move_input(start.0, start.1);
    thread::sleep(Duration::from_millis(cfg.hover_delay_ms));

    win::left_down_at(start.0, start.1);
    thread::sleep(Duration::from_millis(cfg.after_down_ms));

    win::move_input(start.0, start.1);
    thread::sleep(Duration::from_millis(1));

    let mut prev = start;
    for &pt in &path[1..] {
        let target = to_screen_scaled(base_x, base_y, sx, sy, pt);
        for p in bresenham(prev, target) {
            win::move_input(p.0, p.1);
            thread::sleep(Duration::from_millis(cfg.px_delay_ms));
        }
        prev = target;
    }

    win::move_input(prev.0, prev.1);
    thread::sleep(Duration::from_millis(cfg.before_up_ms));

    win::left_up_at(prev.0, prev.1);
    thread::sleep(Duration::from_millis(cfg.stroke_gap_ms));
}

fn draw_stroke(cfg: &Config, base_x: i32, base_y: i32, path: &[Pt]) {
    draw_stroke_scaled(cfg, base_x, base_y, cfg.glyph_w, cfg.char_h, path);
}

fn draw_stroke_full(cfg: &Config, path: &[Pt]) {
    let sx = cfg.area_x2 - cfg.area_x1;
    let sy = cfg.area_y2 - cfg.area_y1;
    draw_stroke_scaled(cfg, cfg.area_x1, cfg.area_y1, sx, sy, path);
}

fn draw_char_at(cfg: &Config, glyphs: &HashMap<char, Glyph>, base_x: i32, base_y: i32, ch: char) {
    if ch == ' ' { return; }
    let key = normalize(ch);
    let Some(g) = glyphs.get(&key) else {
        eprintln!("[WARN] нет глифа для '{}' (ключ '{}')", ch, key);
        return;
    };
    for stroke in g {
        draw_stroke(cfg, base_x, base_y, stroke);
    }
}

fn draw_char_full(cfg: &Config, glyphs: &HashMap<char, Glyph>, ch: char) {
    if ch == ' ' { return; }
    let key = normalize(ch);
    let Some(g) = glyphs.get(&key) else {
        eprintln!("[WARN] нет глифа для '{}' (ключ '{}')", ch, key);
        return;
    };
    for stroke in g {
        draw_stroke_full(cfg, stroke);
    }
}

// ==================== КАРЕТКА ====================

struct Cursor {
    x: i32,
    y: i32,
    line: i32,
}

impl Cursor {
    fn new(cfg: &Config) -> Self {
        Self { x: cfg.area_x1, y: cfg.area_y1, line: 0 }
    }

    fn newline(&mut self, cfg: &Config) {
        self.line += 1;
        self.x = cfg.area_x1;
        self.y = cfg.area_y1 + self.line * (cfg.char_h + cfg.line_gap);
    }

    fn ensure_room(&mut self, cfg: &Config) -> bool {
        let line_right = cfg.area_x1 + cfg.chars_per_line() * cfg.char_w;
        if self.x + cfg.char_w > line_right {
            self.newline(cfg);
        }
        self.line < cfg.lines_count()
    }

    fn advance(&mut self, cfg: &Config) {
        self.x += cfg.char_w;
    }
}

// ==================== MAIN ==================

enum Cmd {
    Draw(char, i32, i32),
    DrawFull(char),
    Reset(i32, i32),
}

fn main() {
    win::boost_timer();
    win::ctrl_release_safe();

    let cfg = Config::load("config.txt");
    println!("[CFG] {:?}", cfg);
    println!(
        "[CFG] область {}×{}, символов в строке = {}, строк = {}",
        cfg.area_x2 - cfg.area_x1,
        cfg.area_y2 - cfg.area_y1,
        cfg.chars_per_line(),
        cfg.lines_count()
    );
    if !cfg.all_area_keys.is_empty() {
        println!(
            "[CFG] all_area_keys = {:?} (эти клавиши рисуют глиф на всю область)",
            cfg.all_area_keys
        );
    } else {
        println!("[CFG] all_area_keys пуст — режим «на всю область» отключён");
    }

    let glyphs = load_glyphs("glyphs.txt");
    println!("Загружено {} глифов из glyphs.txt", glyphs.len());

    let (tx, rx) = mpsc::channel::<Cmd>();

    let cfg_for_thread = cfg.clone();
    thread::spawn(move || {
        while let Ok(cmd) = rx.recv() {
            match cmd {
                Cmd::Draw(ch, x, y) => {
                    draw_char_at(&cfg_for_thread, &glyphs, x, y, ch)
                }
                Cmd::DrawFull(ch) => {
                    draw_char_full(&cfg_for_thread, &glyphs, ch)
                }
                Cmd::Reset(x, y) => {
                    win::move_input(x, y);
                }
            }
        }
    });

    println!("=== Cursor Draw ===");
    println!("Enter       — включить ввод.");
    println!("Shift+Enter — выход из программы.");
    println!("Backspace   — выключить ввод (+ отпустить Ctrl).");
    println!("Tab         — переход на новую строку.");
    println!("Shift       — показать координаты курсора мыши (для настройки).");

    let mut prev_keys = [false; 256];
    let mut active = false;
    let mut cursor = Cursor::new(&cfg);

    loop {
        // --- Esc: ничего не делает (оставлен no-op для совместимости) ---
        if win::escape_was_pressed() {
            println!("[ESC] Esc нажат (ничего не делаем).");
        }

        // --- Backspace: выключить ввод + отпустить Ctrl ---
        if win::backspace_was_pressed() {
            if active {
                active = false;
                println!("[OFF] режим рисования выключен (Backspace).");
            } else {
                println!("[OFF] режим рисования и так выключен.");
            }
            // Отпускаем Ctrl ВСЕГДА — на случай, если он залип.
            win::ctrl_release_safe();
        }

        // --- Enter: Shift+Enter — выход; одиночный Enter — включить ввод ---
        if win::enter_was_pressed() {
            if win::shift_is_down() {
                // Shift+Enter — выход
                win::ctrl_release_safe();
                thread::sleep(Duration::from_millis(30));
                println!("[EXIT] Shift+Enter — выход.");
                std::process::exit(0);
            }

            if !active {
                active = true;
                cursor = Cursor::new(&cfg);
                let _ = tx.send(Cmd::Reset(cursor.x, cursor.y));
                if cfg.use_ctrl { win::ctrl_down(); }
                println!(
                    "[ON]  режим рисования включён{}. Каретка в ({}, {})",
                    if cfg.use_ctrl { ", Ctrl зажат" } else { "" },
                    cursor.x, cursor.y
                );
            } else {
                println!("[ON]  уже включено, Enter игнорирован.");
            }
        }

        // --- Shift (одиночный): печатать координаты курсора ---
        // Проверяем только если НЕ зажат Enter/Backspace и т.п. — чтобы
        // не спамить при Shift+Enter. Но так как Shift+Enter уже обработан
        // выше и вышел из программы, сюда мы попадаем только с одиночным Shift.
        if win::shift_was_pressed() {
            let (mx, my) = win::get_cursor_pos();
            println!("[POS] курсор мыши: x={}, y={}", mx, my);
        }

        // --- Tab: перенос строки (только при активном рисовании) ---
        if active && win::tab_was_pressed() {
            cursor.newline(&cfg);
            if cursor.line >= cfg.lines_count() {
                eprintln!(
                    "[WARN] Tab: строки кончились (line = {}, всего {})",
                    cursor.line, cfg.lines_count()
                );
            } else {
                println!(
                    "[NL]  новая строка #{}, каретка в ({}, {})",
                    cursor.line, cursor.x, cursor.y
                );
            }
        }

        // --- Символы ---
        if active {
            let russian = win::layout_is_russian();
            let new_keys = win::poll_new_keys(&mut prev_keys);
            for vk in new_keys {
                if let Some(ch) = win::vk_to_char(vk, russian) {
                    if cfg.is_all_area_key(ch) {
                        let _ = tx.send(Cmd::DrawFull(ch));
                        println!("[FULL] '{}' рисуется на всю область", ch);
                        continue;
                    }

                    if !cursor.ensure_room(&cfg) {
                        eprintln!("[WARN] область заполнена, символ '{}' пропущен", ch);
                        continue;
                    }

                    let x = cursor.x;
                    let y = cursor.y;
                    let _ = tx.send(Cmd::Draw(ch, x, y));
                    print!("{}", ch);
                    use std::io::Write;
                    std::io::stdout().flush().ok();

                    cursor.advance(&cfg);
                }
            }
        } else {
            let _ = win::poll_new_keys(&mut prev_keys);
        }

        thread::sleep(Duration::from_millis(cfg.poll_interval_ms));
    }
}