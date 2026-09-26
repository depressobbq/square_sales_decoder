use chrono::{Datelike, NaiveDate, NaiveDateTime, NaiveTime, Timelike, Weekday};
use eframe::egui;
use egui_plot::{Bar, BarChart, Plot, PlotBounds};
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

#[derive(Debug, Clone)]
struct Transaction {
    date: NaiveDate,
    time: NaiveTime,
    amount: Decimal,
}

pub fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1100.0, 720.0])
            .with_min_inner_size([800.0, 600.0])
            .with_title("Square Sales Report Analyzer"),
        ..Default::default()
    };

    eframe::run_native(
        "Square Sales Analyzer",
        options,
        Box::new(|_cc| Box::new(SalesApp::default())),
    )
}

struct SalesApp {
    // Loaded Data
    file_path: Option<PathBuf>,
    transactions: Vec<Transaction>,
    grand_total: Decimal,

    // Filter Controls
    selected_weekday: usize, // 0 = All, 1 = Sun, ..., 7 = Sat
    start_hour: u32,
    end_hour: u32,

    // Easter Egg State
    keystroke_buffer: String,
    show_easter_egg_popup: bool,

    // Calculated View Data
    filtered_total: Decimal,
    filtered_count: usize,
    hourly_sales: [f64; 24],
}

impl Default for SalesApp {
    fn default() -> Self {
        Self {
            file_path: None,
            transactions: Vec::new(),
            grand_total: Decimal::ZERO,
            selected_weekday: 1, // Default to Sunday
            start_hour: 6,       // Default 6 AM
            end_hour: 8,         // Default 8 AM
            keystroke_buffer: String::new(),
            show_easter_egg_popup: false,
            filtered_total: Decimal::ZERO,
            filtered_count: 0,
            hourly_sales: [0.0; 24],
        }
    }
}

impl SalesApp {
    fn recompute_filters(&mut self) {
        let target_day = match self.selected_weekday {
            1 => Some(Weekday::Sun),
            2 => Some(Weekday::Mon),
            3 => Some(Weekday::Tue),
            4 => Some(Weekday::Wed),
            5 => Some(Weekday::Thu),
            6 => Some(Weekday::Fri),
            7 => Some(Weekday::Sat),
            _ => None,
        };

        let start_t = NaiveTime::from_hms_opt(self.start_hour, 0, 0).unwrap();
        let end_t = NaiveTime::from_hms_opt(self.end_hour, 59, 59).unwrap();

        let mut total = Decimal::ZERO;
        let mut count = 0usize;
        let mut hourly = [0.0f64; 24];

        for t in &self.transactions {
            if let Some(day) = target_day {
                if t.date.weekday() != day {
                    continue;
                }
            }

            // Track hourly totals for chart
            let h = t.time.hour() as usize;
            if h < 24 {
                hourly[h] += t.amount.to_f64().unwrap_or(0.0);
            }

            // Filter target time window
            if t.time >= start_t && t.time <= end_t {
                total += t.amount;
                count += 1;
            }
        }

        self.filtered_total = total;
        self.filtered_count = count;
        self.hourly_sales = hourly;
    }

    fn load_csv(&mut self, path: PathBuf) {
        if let Ok(file) = File::open(&path) {
            let mut rdr = csv::ReaderBuilder::new().flexible(true).from_reader(file);
            let headers = match rdr.headers() {
                Ok(h) => h.clone(),
                Err(_) => return,
            };

            let date_idx = find_column(&headers, &["date", "timestamp", "created at"]).unwrap_or(0);
            let time_idx = find_column(&headers, &["time"]);
            let amount_idx = find_column(
                &headers,
                &[
                    "total collected",
                    "total",
                    "net sales",
                    "gross sales",
                    "amount",
                ],
            )
            .unwrap_or(1);

            let mut list = Vec::new();
            let mut grand = Decimal::ZERO;

            for result in rdr.records() {
                let record = match result {
                    Ok(r) => r,
                    Err(_) => continue,
                };
                let raw_date_str = record.get(date_idx).unwrap_or("").trim();
                if raw_date_str.is_empty() {
                    continue;
                }

                let (record_date, record_time) = match parse_datetime(raw_date_str) {
                    Ok(dt) => (dt.date(), dt.time()),
                    Err(_) => {
                        let parsed_date = match parse_flexible_date(raw_date_str) {
                            Some(d) => d,
                            None => continue,
                        };
                        let parsed_time = if let Some(t_idx) = time_idx {
                            let raw_time_str = record.get(t_idx).unwrap_or("").trim();
                            parse_flexible_time(raw_time_str)
                                .unwrap_or(NaiveTime::from_hms_opt(0, 0, 0).unwrap())
                        } else {
                            NaiveTime::from_hms_opt(0, 0, 0).unwrap()
                        };
                        (parsed_date, parsed_time)
                    }
                };

                let raw_amount = record.get(amount_idx).unwrap_or("0");
                let cleaned = raw_amount
                    .replace('$', "")
                    .replace(',', "")
                    .trim()
                    .to_string();

                if let Ok(amount) = cleaned.parse::<Decimal>() {
                    grand += amount;
                    list.push(Transaction {
                        date: record_date,
                        time: record_time,
                        amount,
                    });
                }
            }

            self.transactions = list;
            self.grand_total = grand;
            self.file_path = Some(path);
            self.recompute_filters();
        }
    }

    fn export_filtered_csv(&self, path: PathBuf) {
        if let Ok(mut file) = File::create(path) {
            let _ = writeln!(file, "Date,Time,Weekday,Amount");
            let target_day = match self.selected_weekday {
                1 => Some(Weekday::Sun),
                2 => Some(Weekday::Mon),
                3 => Some(Weekday::Tue),
                4 => Some(Weekday::Wed),
                5 => Some(Weekday::Thu),
                6 => Some(Weekday::Fri),
                7 => Some(Weekday::Sat),
                _ => None,
            };
            let start_t = NaiveTime::from_hms_opt(self.start_hour, 0, 0).unwrap();
            let end_t = NaiveTime::from_hms_opt(self.end_hour, 59, 59).unwrap();

            for t in &self.transactions {
                if let Some(day) = target_day {
                    if t.date.weekday() != day {
                        continue;
                    }
                }
                if t.time >= start_t && t.time <= end_t {
                    let _ = writeln!(
                        file,
                        "{},{},{:?},{:.2}",
                        t.date.format("%Y-%m-%d"),
                        t.time.format("%H:%M:%S"),
                        t.date.weekday(),
                        t.amount
                    );
                }
            }
        }
    }
}

impl eframe::App for SalesApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // --- 1. Global Key Listener for "bingbong" Easter Egg ---
        ctx.input(|i| {
            for event in &i.events {
                if let egui::Event::Key {
                    key, pressed: true, ..
                } = event
                {
                    let key_str = format!("{:?}", key).to_lowercase();
                    if key_str.len() == 1 {
                        self.keystroke_buffer.push_str(&key_str);
                        if self.keystroke_buffer.len() > 20 {
                            self.keystroke_buffer
                                .drain(..self.keystroke_buffer.len() - 20);
                        }
                        if self.keystroke_buffer.ends_with("bingbong") {
                            self.show_easter_egg_popup = true;
                            self.keystroke_buffer.clear();
                        }
                    }
                }
            }

            // Drag and Drop Handler
            if !i.raw.dropped_files.is_empty() {
                if let Some(path) = i.raw.dropped_files[0].path.clone() {
                    self.load_csv(path);
                }
            }
        });

        // --- 2. Easter Egg Modal Pop-Up ---
        if self.show_easter_egg_popup {
            egui::Window::new("MESSAGE FROM SYSTEM")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.add_space(10.0);
                        ui.colored_label(
                            egui::Color32::RED,
                            egui::RichText::new("fuck your life").size(28.0).strong(),
                        );
                        ui.add_space(15.0);
                        if ui.button(" OK ").clicked() {
                            self.show_easter_egg_popup = false;
                        }
                        ui.add_space(5.0);
                    });
                });
        }

        // --- 3. Top Header Bar ---
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("📊 Square Sales Report Analyzer");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("📂 Open CSV File").clicked() {
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("CSV Files", &["csv"])
                            .pick_file()
                        {
                            self.load_csv(path);
                        }
                    }
                });
            });
        });

        // --- 4. Sidebar Controls ---
        egui::SidePanel::left("left_panel")
            .resizable(false)
            .default_width(280.0)
            .show(ctx, |ui| {
                ui.heading("Filters & Options");
                ui.add_space(15.0);

                ui.label("1. Select Target Day:");
                let days = [
                    "All Days",
                    "Sunday",
                    "Monday",
                    "Tuesday",
                    "Wednesday",
                    "Thursday",
                    "Friday",
                    "Saturday",
                ];
                for (idx, day_name) in days.iter().enumerate() {
                    if ui
                        .selectable_value(&mut self.selected_weekday, idx, *day_name)
                        .clicked()
                    {
                        self.recompute_filters();
                    }
                }

                ui.add_space(15.0);
                ui.label("2. Select Time Window (24h):");
                ui.horizontal(|ui| {
                    ui.label("Start Hour:");
                    if ui
                        .add(egui::DragValue::new(&mut self.start_hour).clamp_range(0..=23))
                        .changed()
                    {
                        self.recompute_filters();
                    }
                    ui.label("End Hour:");
                    if ui
                        .add(egui::DragValue::new(&mut self.end_hour).clamp_range(0..=23))
                        .changed()
                    {
                        self.recompute_filters();
                    }
                });

                ui.add_space(20.0);
                ui.separator();
                ui.add_space(10.0);

                if ui.button("💾 Export Window CSV").clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .set_file_name("filtered_sales.csv")
                        .save_file()
                    {
                        self.export_filtered_csv(path);
                    }
                }
            });

        // --- 5. Main Display Panel ---
        egui::CentralPanel::default().show(ctx, |ui| {
            if self.transactions.is_empty() {
                ui.centered_and_justified(|ui| {
                    ui.heading("Drag & Drop your Square CSV file anywhere here to begin");
                });
            } else {
                ui.columns(3, |cols| {
                    cols[0].metric("Grand Total Sales", format!("${:.2}", self.grand_total));
                    cols[1].metric("Window Total Sales", format!("${:.2}", self.filtered_total));

                    let pct = if self.grand_total > Decimal::ZERO {
                        (self.filtered_total / self.grand_total) * Decimal::from(100)
                    } else {
                        Decimal::ZERO
                    };
                    cols[2].metric("Share of Store Total", format!("{:.2}%", pct));
                });

                ui.add_space(15.0);
                ui.separator();
                ui.add_space(10.0);

                ui.heading("Hourly Sales Breakdown (Hover bar to inspect)");

                let mut bars = Vec::new();
                for h in 0..24 {
                    let amount = self.hourly_sales[h];
                    let is_in_window = h as u32 >= self.start_hour && h as u32 <= self.end_hour;

                    let fill_color = if is_in_window {
                        egui::Color32::LIGHT_BLUE
                    } else {
                        egui::Color32::DARK_GRAY
                    };

                    bars.push(
                        Bar::new(h as f64 + 0.5, amount)
                            .width(0.8)
                            .fill(fill_color)
                            .name(format!("{:02}:00 - ${:.2}", h, amount)),
                    );
                }

                let chart = BarChart::new(bars).name("Hourly Volume");

                Plot::new("hourly_plot")
                    .legend(egui_plot::Legend::default())
                    .height(380.0)
                    .allow_zoom(false)
                    .allow_drag(false)
                    .show_x(true)
                    .show_y(true)
                    .show(ui, |plot_ui| {
                        plot_ui.bar_chart(chart);
                        plot_ui.set_plot_bounds(PlotBounds::from_min_max(
                            [0.0, 0.0],
                            [
                                24.0,
                                self.hourly_sales.iter().cloned().fold(0.0, f64::max) * 1.15,
                            ],
                        ));
                    });
            }
        });
    }
}

trait MetricUi {
    fn metric(&mut self, label: &str, value: String);
}

impl MetricUi for egui::Ui {
    fn metric(&mut self, label: &str, value: String) {
        self.group(|ui| {
            ui.vertical_centered(|ui| {
                ui.label(label);
                ui.heading(value);
            });
        });
    }
}

fn find_column(headers: &csv::StringRecord, targets: &[&str]) -> Option<usize> {
    for target in targets {
        for (i, header) in headers.iter().enumerate() {
            if header.trim().to_lowercase() == *target {
                return Some(i);
            }
        }
    }
    None
}

fn parse_flexible_time(s: &str) -> Option<NaiveTime> {
    let clean = s.trim();
    let formats = ["%I:%M %p", "%I:%M%p", "%I %p", "%H:%M:%S", "%H:%M"];
    for fmt in &formats {
        if let Ok(t) = NaiveTime::parse_from_str(clean, fmt) {
            return Some(t);
        }
    }
    None
}

fn parse_flexible_date(s: &str) -> Option<NaiveDate> {
    let clean = s.trim();
    if clean.is_empty() {
        return None;
    }
    let formats = ["%m/%d/%Y", "%m/%d/%y", "%Y-%m-%d", "%m-%d-%Y"];
    for fmt in &formats {
        if let Ok(d) = NaiveDate::parse_from_str(clean, fmt) {
            return Some(d);
        }
    }
    None
}

fn parse_datetime(s: &str) -> Result<NaiveDateTime, ()> {
    let formats = [
        "%Y-%m-%d %H:%M:%S",
        "%Y-%m-%dT%H:%M:%S",
        "%m/%d/%Y %H:%M:%S",
        "%m/%d/%y %H:%M:%S",
        "%m/%d/%Y %I:%M:%S %p",
        "%m/%d/%y %I:%M:%S %p",
        "%m/%d/%Y %I:%M %p",
    ];
    for fmt in &formats {
        if let Ok(dt) = NaiveDateTime::parse_from_str(s, fmt) {
            return Ok(dt);
        }
    }
    Err(())
}
