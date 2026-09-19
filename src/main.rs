#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui;
use serde::Deserialize;
use std::io::{BufRead, BufReader};
use std::os::windows::process::CommandExt;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{Receiver, TryRecvError, channel};
use std::sync::{Arc, Mutex};
use std::thread;

const CREATE_NO_WINDOW: u32 = 0x08000000;

// --- Color palettes, chosen at startup based on settings.jsonc ---
mod theme {
    use eframe::egui::Color32;

    #[derive(Clone, Copy)]
    pub struct Palette {
        pub base: Color32,
        pub text: Color32,
        pub subtext: Color32,
        pub yellow: Color32,
        pub red: Color32,
        pub green: Color32,
        pub blue: Color32,
        pub lavender: Color32,
    }

    impl Palette {
        pub fn latte() -> Self {
            Self {
                base: Color32::from_rgb(239, 241, 245),
                text: Color32::from_rgb(76, 79, 105),
                subtext: Color32::from_rgb(92, 95, 119),
                yellow: Color32::from_rgb(223, 142, 29),
                red: Color32::from_rgb(210, 15, 57),
                green: Color32::from_rgb(64, 160, 43),
                blue: Color32::from_rgb(30, 102, 245),
                lavender: Color32::from_rgb(114, 135, 253),
            }
        } // Latte

        pub fn frappe() -> Self {
            Self {
                base: Color32::from_rgb(48, 52, 70),
                text: Color32::from_rgb(198, 208, 245),
                subtext: Color32::from_rgb(181, 191, 226),
                yellow: Color32::from_rgb(229, 200, 144),
                red: Color32::from_rgb(231, 130, 132),
                green: Color32::from_rgb(166, 209, 137),
                blue: Color32::from_rgb(140, 170, 238),
                lavender: Color32::from_rgb(186, 187, 241),
            }
        } // Frappe

        pub fn macchiato() -> Self {
            Self {
                base: Color32::from_rgb(36, 39, 58),
                text: Color32::from_rgb(202, 211, 245),
                subtext: Color32::from_rgb(184, 192, 224),
                yellow: Color32::from_rgb(238, 212, 159),
                red: Color32::from_rgb(237, 135, 150),
                green: Color32::from_rgb(166, 218, 149),
                blue: Color32::from_rgb(138, 173, 244),
                lavender: Color32::from_rgb(183, 189, 248),
            }
        } // Macchiato

        pub fn mocha() -> Self {
            Self {
                base: Color32::from_rgb(30, 30, 46),
                text: Color32::from_rgb(205, 214, 244),
                subtext: Color32::from_rgb(186, 194, 222),
                yellow: Color32::from_rgb(249, 226, 175),
                red: Color32::from_rgb(243, 139, 168),
                green: Color32::from_rgb(166, 227, 161),
                blue: Color32::from_rgb(137, 180, 250),
                lavender: Color32::from_rgb(180, 190, 254),
            } // Mocha
        }

        // Unknown or missing names fall back to Frappé.
        pub fn from_name(name: &str) -> Self {
            match name.to_lowercase().as_str() {
                "latte" => Self::latte(),
                "macchiato" => Self::macchiato(),
                "mocha" => Self::mocha(),
                _ => Self::frappe(),
            }
        }
    }
}

// --- settings.jsonc structure ---
#[derive(Deserialize)]
#[serde(default)]
struct SettingsFile {
    theme: String,
    font: String,
}

impl Default for SettingsFile {
    fn default() -> Self {
        Self {
            theme: "frappe".to_string(),
            font: String::new(),
        }
    }
}

fn settings_dir() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."))
}

fn settings_path() -> PathBuf {
    let mut path = settings_dir();
    path.push("settings.jsonc");
    path
}

const DEFAULT_SETTINGS: &str = r#"{
    // This is the settings file for CozyMDT settings
    // Available themes: "latte", "frappe", "macchiato, "mocha"
    "theme": "frappe",

    // Path to a custom .ttf font file (use the full path, e.g. "C:\\Fonts\\myfont.ttf").
    // Leave empty to use the default bundled font (JetBrainsMono Nerd Font)
    "font": ""
}
"#;

// Loads settings.jsonc, creating it with defaults if missing. Returns the
// parsed settings plus any warnings to show the user (e.g. if the file
// couldn't be parsed, we fall back to defaults instead of crashing).
fn load_or_create_settings() -> (SettingsFile, Vec<String>) {
    let mut warnings = Vec::new();
    let path = settings_path();

    if !path.exists() {
        if let Err(e) = std::fs::write(&path, DEFAULT_SETTINGS) {
            warnings.push(format!("Could not create settings file: {}", e));
        }
    }

    match std::fs::read_to_string(&path) {
        Ok(content) => match json5::from_str::<SettingsFile>(&content) {
            Ok(settings) => (settings, warnings),
            Err(e) => {
                warnings.push(format!(
                    "Could not parse settings.jsonc ({}), using defaults",
                    e
                ));
                (SettingsFile::default(), warnings)
            }
        },
        Err(e) => {
            warnings.push(format!(
                "Could not read settings.jsonc ({}), using defaults",
                e
            ));
            (SettingsFile::default(), warnings)
        }
    }
}

// Loads the custom font's bytes from disk, if one is set in settings.
// Returns None (fall back to the bundled font) if the field is empty
// or the file can't be read.
fn load_custom_font(settings: &SettingsFile) -> (Option<Vec<u8>>, Vec<String>) {
    let mut warnings = Vec::new();

    if settings.font.trim().is_empty() {
        return (None, warnings);
    }

    match std::fs::read(&settings.font) {
        Ok(bytes) => (Some(bytes), warnings),
        Err(e) => {
            warnings.push(format!(
                "Could not load custom font '{}' ({}), using default font",
                settings.font, e
            ));
            (None, warnings)
        }
    }
}

fn setup_fonts(ctx: &egui::Context, custom_font_bytes: Option<Vec<u8>>) {
    let mut fonts = egui::FontDefinitions::default();

    // Use the custom font from settings.jsonc if one loaded successfully,
    // otherwise fall back to the font bundled inside the executable
    let bytes = custom_font_bytes
        .unwrap_or_else(|| include_bytes!("../assets/jetbrains-mono-nerd.ttf").to_vec());

    fonts
        .font_data
        .insert("cozymdt_font".to_owned(), egui::FontData::from_owned(bytes));

    fonts
        .families
        .entry(egui::FontFamily::Monospace)
        .or_default()
        .insert(0, "cozymdt_font".to_owned());

    ctx.set_fonts(fonts);
}

enum Shell {
    PowerShell,
    Cmd,
    Bash,
    CozyT,
}

impl Shell {
    fn program_and_flag(&self) -> (String, &str) {
        match self {
            Shell::PowerShell => ("powershell".to_string(), "-Command"),
            Shell::Cmd => ("cmd".to_string(), "/C"),
            Shell::Bash => (find_bash(), "-c"),
            Shell::CozyT => unreachable!("CozyT has no external program"),
        }
    }

    fn name(&self) -> &str {
        match self {
            Shell::PowerShell => "powershell",
            Shell::Cmd => "cmd",
            Shell::Bash => "bash",
            Shell::CozyT => "cozyt",
        }
    }
}

fn strip_unc_prefix(path: PathBuf) -> PathBuf {
    let s = path.to_string_lossy();
    match s.strip_prefix(r"\\?\") {
        Some(stripped) => PathBuf::from(stripped),
        None => path,
    }
}

// Git for Windows doesn't add bash.exe to the system PATH by default —
// only Git Bash's own sessions have it. So we check the common install
// locations directly instead of relying on PATH.
fn find_bash() -> String {
    let candidates = [
        r"C:\Program Files\Git\bin\bash.exe",
        r"C:\Program Files (x86)\Git\bin\bash.exe",
    ];

    for candidate in candidates {
        if std::path::Path::new(candidate).exists() {
            return candidate.to_string();
        }
    }

    "bash".to_string()
}

// Decides the color for a command's first word — green for "exit",
// yellow for everything else (shell names, CozyT commands, etc.)
fn first_word_color(word: &str, palette: &theme::Palette) -> egui::Color32 {
    if word == "exit" {
        palette.green
    } else {
        palette.yellow
    }
}

enum HistoryLine {
    Banner(String),
    Command { prompt: String, input: String },
    Output(String),
    Error(String),
    Info(String),
}

struct CozyMdtApp {
    current_shell: Shell,
    current_dir: PathBuf,
    input: String,
    history: Vec<HistoryLine>,
    focus_input: bool,
    running_child: Arc<Mutex<Option<Child>>>,
    output_rx: Option<Receiver<String>>,
    is_running: bool,
    palette: theme::Palette,
}

impl CozyMdtApp {
    fn new(palette: theme::Palette, startup_warnings: Vec<String>) -> Self {
        let mut history = vec![
            HistoryLine::Banner("=== CozyMDT ===".to_string()),
            HistoryLine::Banner("  By FRANORDE   ".to_string()),
        ];

        for warning in startup_warnings {
            history.push(HistoryLine::Error(warning));
        }

        Self {
            current_shell: Shell::CozyT,
            current_dir: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            input: String::new(),
            history,
            focus_input: true,
            running_child: Arc::new(Mutex::new(None)),
            output_rx: None,
            is_running: false,
            palette,
        }
    }

    fn log(&mut self, line: impl Into<String>) {
        self.history.push(HistoryLine::Output(line.into()));
    }

    fn log_error(&mut self, line: impl Into<String>) {
        self.history.push(HistoryLine::Error(line.into()));
    }

    fn log_info(&mut self, line: impl Into<String>) {
        self.history.push(HistoryLine::Info(line.into()));
    }

    fn log_command(&mut self, input: &str) {
        self.history.push(HistoryLine::Command {
            prompt: self.prompt(),
            input: input.to_string(),
        });
    }

    fn prompt(&self) -> String {
        format!(
            "{} [{}]> ",
            self.current_dir.display(),
            self.current_shell.name()
        )
    }

    fn handle_cozyt_command(&mut self, input: &str) -> bool {
        match input {
            "help" => {
                self.log("CozyT built-in commands:");
                self.log("  help     - show this message");
                self.log("  version  - show CozyMDT version");
                self.log("  settings - open CozyMDT settings file (theme, font)");
                self.log("  cd <dir> - change current directory");
                true
            }
            "version" => {
                self.log("CozyMDT v0.1.0");
                true
            }
            "settings" => {
                let path = settings_path();

                if !path.exists() {
                    if let Err(e) = std::fs::write(&path, DEFAULT_SETTINGS) {
                        self.log_error(format!("Could not create settings file: {}", e));
                        return true;
                    }
                    self.log_info(format!(
                        "Created default settings file at {}",
                        path.display()
                    ));
                }

                if let Err(e) = open::that(&path) {
                    self.log_error(format!("Could not open settings file: {}", e));
                } else {
                    self.log_info("Restart CozyMDT after saving to apply theme/font changes.");
                }
                true
            }
            _ => false,
        }
    }

    fn handle_cd(&mut self, target: &str) {
        let target = target.trim();

        let new_path = if target.is_empty() {
            dirs::home_dir().unwrap_or_else(|| self.current_dir.clone())
        } else {
            self.current_dir.join(target)
        };

        match std::fs::canonicalize(&new_path) {
            Ok(resolved) => {
                self.current_dir = strip_unc_prefix(resolved);
            }
            Err(e) => {
                self.log_error(format!("cd: {}: {}", target, e));
            }
        }
    }

    fn run_external(&mut self, program: &str, args: Vec<String>) {
        let (tx, rx) = channel::<String>();
        self.output_rx = Some(rx);
        self.is_running = true;

        let program = program.to_string();
        let cwd = self.current_dir.clone();
        let child_slot = Arc::clone(&self.running_child);

        thread::spawn(move || {
            let mut cmd = Command::new(&program);
            cmd.args(&args)
                .current_dir(&cwd)
                .stdin(Stdio::null())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .creation_flags(CREATE_NO_WINDOW);

            match cmd.spawn() {
                Ok(mut child) => {
                    let stdout = child.stdout.take();
                    let stderr = child.stderr.take();

                    *child_slot.lock().unwrap() = Some(child);

                    let mut readers = Vec::new();

                    if let Some(stdout) = stdout {
                        let tx_out = tx.clone();
                        readers.push(thread::spawn(move || {
                            for line in BufReader::new(stdout).lines().flatten() {
                                let _ = tx_out.send(line);
                            }
                        }));
                    }
                    if let Some(stderr) = stderr {
                        let tx_err = tx.clone();
                        readers.push(thread::spawn(move || {
                            for line in BufReader::new(stderr).lines().flatten() {
                                let _ = tx_err.send(line);
                            }
                        }));
                    }

                    for r in readers {
                        let _ = r.join();
                    }

                    if let Some(mut child) = child_slot.lock().unwrap().take() {
                        let _ = child.wait();
                    }
                }
                Err(e) => {
                    let _ = tx.send(format!("Error executing command: {}", e));
                }
            }
        });
    }

    fn run_command(&mut self, input: &str) {
        self.log_command(input);

        if input.is_empty() {
            return;
        }

        if let Some(target) = input.strip_prefix("cd ") {
            self.handle_cd(target);
            return;
        }
        if input == "cd" {
            self.handle_cd("");
            return;
        }

        if let Some(target) = input.strip_prefix("switch ") {
            self.current_shell = match target.trim() {
                "powershell" | "ps" => Shell::PowerShell,
                "cmd" => Shell::Cmd,
                "bash" => Shell::Bash,
                "cozyt" => Shell::CozyT,
                other => {
                    self.log_error(format!("Unknown shell: {}", other));
                    return;
                }
            };
            self.log_info(format!("Switched to {}", self.current_shell.name()));
            return;
        }

        if let Shell::CozyT = self.current_shell {
            if !self.handle_cozyt_command(input) {
                self.log_error(format!("Unknown CozyT command: '{}' (try 'help')", input));
            }
            return;
        }

        let (program, flag) = self.current_shell.program_and_flag();
        let flag = flag.to_string();
        self.run_external(&program, vec![flag, input.to_string()]);
    }
}

fn draw_history_line(ui: &mut egui::Ui, line: &HistoryLine, palette: &theme::Palette) {
    match line {
        HistoryLine::Banner(text) => {
            ui.colored_label(palette.lavender, text);
        }
        HistoryLine::Output(text) => {
            ui.colored_label(palette.text, text);
        }
        HistoryLine::Error(text) => {
            ui.colored_label(palette.red, text);
        }
        HistoryLine::Info(text) => {
            ui.colored_label(palette.blue, text);
        }
        HistoryLine::Command { prompt, input } => {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                ui.colored_label(palette.subtext, prompt);

                let mut parts = input.splitn(2, ' ');
                if let Some(first_word) = parts.next() {
                    ui.colored_label(first_word_color(first_word, palette), first_word);
                }
                if let Some(rest) = parts.next() {
                    ui.colored_label(palette.text, format!(" {}", rest));
                }
            });
        }
    }
}

fn layout_command_input(
    ui: &egui::Ui,
    text: &str,
    wrap_width: f32,
    palette: &theme::Palette,
) -> std::sync::Arc<egui::Galley> {
    let mut job = egui::text::LayoutJob::default();
    let font_id = egui::TextStyle::Monospace.resolve(ui.style());

    let mut parts = text.splitn(2, ' ');
    let first_word = parts.next().unwrap_or("");
    let rest = parts.next();

    if !first_word.is_empty() {
        job.append(
            first_word,
            0.0,
            egui::TextFormat {
                font_id: font_id.clone(),
                color: first_word_color(first_word, palette),
                ..Default::default()
            },
        );
    }

    if let Some(rest) = rest {
        job.append(
            &format!(" {}", rest),
            0.0,
            egui::TextFormat {
                font_id,
                color: palette.text,
                ..Default::default()
            },
        );
    }

    job.wrap.max_width = wrap_width;
    ui.fonts(|f| f.layout_job(job))
}

impl eframe::App for CozyMdtApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut new_lines = Vec::new();
        let mut disconnected = false;
        if let Some(rx) = &self.output_rx {
            loop {
                match rx.try_recv() {
                    Ok(line) => new_lines.push(line),
                    Err(TryRecvError::Empty) => break,
                    Err(TryRecvError::Disconnected) => {
                        disconnected = true;
                        break;
                    }
                }
            }
        }
        for line in new_lines {
            self.log(line);
        }
        if disconnected {
            self.output_rx = None;
            self.is_running = false;
        }

        if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::C)) && self.is_running {
            let pid = self.running_child.lock().unwrap().as_ref().map(|c| c.id());

            if let Some(pid) = pid {
                let _ = Command::new("taskkill")
                    .args(["/PID", &pid.to_string(), "/T", "/F"])
                    .creation_flags(CREATE_NO_WINDOW)
                    .output();
            }

            self.log_error("^C");
        }

        let palette = self.palette;

        egui::CentralPanel::default()
            .frame(egui::Frame::default().fill(palette.base).inner_margin(10.0))
            .show(ctx, |ui| {
                ui.style_mut().override_text_style = Some(egui::TextStyle::Monospace);

                egui::ScrollArea::vertical()
                    .stick_to_bottom(true)
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        for line in &self.history {
                            draw_history_line(ui, line, &palette);
                        }

                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 0.0;
                            ui.colored_label(palette.subtext, self.prompt());

                            let mut layouter = |ui: &egui::Ui, text: &str, wrap_width: f32| {
                                layout_command_input(ui, text, wrap_width, &palette)
                            };

                            let text_edit = egui::TextEdit::singleline(&mut self.input)
                                .frame(false)
                                .desired_width(f32::INFINITY)
                                .text_color(palette.text)
                                .layouter(&mut layouter)
                                .interactive(!self.is_running);

                            let response = ui.add(text_edit);

                            if self.focus_input && !self.is_running {
                                response.request_focus();
                                self.focus_input = false;
                            }

                            if !self.is_running
                                && response.lost_focus()
                                && ui.input(|i| i.key_pressed(egui::Key::Enter))
                            {
                                let input = std::mem::take(&mut self.input);
                                let trimmed = input.trim().to_string();

                                if trimmed == "exit" {
                                    std::process::exit(0);
                                }

                                self.run_command(&trimmed);
                                self.focus_input = true;
                            }
                        });
                    });
            });

        ctx.request_repaint();
    }
}

fn main() -> eframe::Result<()> {
    let (settings, mut warnings) = load_or_create_settings();
    let palette = theme::Palette::from_name(&settings.theme);

    let (custom_font, font_warnings) = load_custom_font(&settings);
    warnings.extend(font_warnings);

    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "CozyMDT",
        options,
        Box::new(move |cc| {
            setup_fonts(&cc.egui_ctx, custom_font);
            Ok(Box::new(CozyMdtApp::new(palette, warnings)))
        }),
    )
}
