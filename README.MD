# Square Sales Report Analyzer

A lightweight, high-performance desktop application built with **Rust** and **egui** to analyze CSV export reports from Square POS. 

It provides instant visual breakdowns, custom time-window filtering, and store share percentages without needing external spreadsheet software or cloud connections.

---

## Features

- 📁 **CSV Import & Drag-and-Drop:** Easily load Square sales reports directly from your local system.
- ⏱️ **Time Window Filtering:** Filter transactions by specific days of the week and custom hourly ranges (24-hour format).
- 📊 **Interactive Hourly Visualizations:** Hover over hourly sales bars to inspect volume dynamically.
- 📈 **Key Metrics At a Glance:** Real-time metrics for Grand Total Sales, Filtered Window Sales, and Store Share Percentage.
- 💾 **Filtered CSV Export:** Export processed time-window subsets directly back into CSV format.
- 🔒 **Privacy-First:** Processes all data locally on-device with zero external network tracking.

---

## Technical Stack

- **GUI Framework:** [`eframe`](https://github.com/emilk/egui/tree/master/crates/eframe) / [`egui`](https://github.com/emilk/egui)
- **Plotting Engine:** [`egui_plot`](https://github.com/emilk/egui/tree/master/crates/egui_plot)
- **CSV Parsing:** [`csv`](https://docs.rs/csv)
- **Date & Time:** [`chrono`](https://docs.rs/chrono)
- **Precise Financial Math:** [`rust_decimal`](https://docs.rs/rust_decimal)
- **Native File Dialogs:** [`rfd`](https://docs.rs/rfd)

---

## Getting Started

### Prerequisites

- [Rust Toolchain](https://www.rust-lang.org/tools/install) (Edition 2021 or newer)

### Building & Running Locally (Linux / macOS / Windows)

```bash
# Clone the repository
git clone [https://github.com/YOUR_USERNAME/square-sales-analyzer.git](https://github.com/YOUR_USERNAME/square-sales-analyzer.git)
cd square-sales-analyzer

# Run in development mode
cargo run --release
