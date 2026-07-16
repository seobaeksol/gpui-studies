use std::{collections::VecDeque, time::Duration};

use battery::units::ratio::percent;
use gpui::{
    App, Bounds, Context, Entity, Hsla, Window, WindowBounds, WindowOptions, div,
    linear_color_stop, linear_gradient, prelude::*, px, size,
};
use gpui_component::chart::AreaChart;
use gpui_component::table::DataTable;
use gpui_component::{ActiveTheme, Sizable, ThemeMode};
use gpui_component::{
    IconName, h_flex,
    table::{Column, ColumnSort, TableDelegate, TableState},
    v_flex,
};
use gpui_platform::application;
use smol::Timer;
use sysinfo::{Disks, Pid, System};

const INTERVAL: Duration = Duration::from_millis(500);
const MAX_DATA_POINTS: usize = 120;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum MonitorTab {
    #[default]
    System = 0,
    Processes = 1,
}

impl MonitorTab {
    fn from_index(idx: usize) -> Self {
        match idx {
            0 => Self::System,
            1 => Self::Processes,
            _ => Self::System,
        }
    }
}

#[derive(Clone)]
struct MetricPoint {
    time: String,
    cpu: f64,
    memory: f64,
}

#[derive(Clone)]
struct ProcessInfo {
    pid: Pid,
    name: String,
    cpu_usage: f32,
    memory: u64,
}

#[derive(Clone)]
struct DiskInfo {
    name: String,
    total: u64,
    used: u64,
}

#[derive(Clone)]
struct BatteryInfo {
    model: String,
    icon: IconName,
    percentage: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum ProcessSortField {
    Pid,
    Name,
    #[default]
    Cpu,
    Memory,
}

struct ProcessTableDelegate {
    processes: Vec<ProcessInfo>,
    columns: Vec<Column>,
    sort_field: ProcessSortField,
    sort_order: ColumnSort,
}

impl ProcessTableDelegate {
    fn new() -> Self {
        Self {
            processes: Vec::new(),
            columns: vec![
                Column::new("pid", "PID").width(70.).sortable(),
                Column::new("name", "Name").width(380.).sortable(),
                Column::new("cpu", "CPU %")
                    .width(80.)
                    .sortable()
                    .sort(ColumnSort::Descending),
                Column::new("memory", "Memory").width(100.).sortable(),
            ],
            sort_field: ProcessSortField::Cpu,
            sort_order: ColumnSort::Descending,
        }
    }

    fn update_processes(&mut self, sys: &System) {
        self.processes = sys
            .processes()
            .iter()
            .map(|(pid, process)| ProcessInfo {
                pid: *pid,
                name: process.name().to_string_lossy().to_string(),
                cpu_usage: process.cpu_usage(),
                memory: process.memory(),
            })
            .collect();

        self.sort_processes();
    }

    fn sort_processes(&mut self) {
        let is_descending = matches!(self.sort_order, ColumnSort::Descending);

        self.processes.sort_by(|a, b| {
            let cmp = match self.sort_field {
                ProcessSortField::Pid => a.pid.as_u32().cmp(&b.pid.as_u32()),
                ProcessSortField::Name => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
                ProcessSortField::Cpu => a.cpu_usage.total_cmp(&b.cpu_usage),
                ProcessSortField::Memory => a.memory.cmp(&b.memory),
            };

            if is_descending { cmp.reverse() } else { cmp }
        });

        // Keep top 200 processes
        self.processes.truncate(200);
    }
}

impl TableDelegate for ProcessTableDelegate {
    fn columns_count(&self, cx: &App) -> usize {
        self.columns.len()
    }

    fn rows_count(&self, cx: &App) -> usize {
        self.processes.len()
    }

    fn column(&self, col_ix: usize, cx: &App) -> Column {
        self.columns[col_ix].clone()
    }

    fn render_td(
        &mut self,
        row_ix: usize,
        col_ix: usize,
        window: &mut Window,
        cx: &mut Context<gpui_component::table::TableState<Self>>,
    ) -> impl IntoElement {
        let Some(process) = self.processes.get(row_ix) else {
            return div().into_any_element();
        };

        match col_ix {
            0 => div()
                .text_xs()
                .text_color(cx.theme().muted_foreground)
                .child(format!("{}", process.pid))
                .into_any_element(),
            1 => div()
                .text_sm()
                .text_color(cx.theme().foreground)
                .truncate()
                .child(process.name.clone())
                .into_any_element(),
            2 => div()
                .text_xs()
                .text_color(if process.cpu_usage > 50.0 {
                    cx.theme().red
                } else if process.cpu_usage > 20.0 {
                    cx.theme().yellow
                } else {
                    cx.theme().blue
                })
                .into_any_element(),
            3 => div()
                .text_xs()
                .text_color(cx.theme().green)
                .child(format_bytes(process.memory))
                .into_any_element(),
            _ => div().into_any_element(),
        }
    }
}

fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

struct SystemMonitor {
    sys: System,
    disks: Disks,
    data: VecDeque<MetricPoint>,
    time_index: usize,
    active_tab: MonitorTab,
    process_table: Entity<TableState<ProcessTableDelegate>>,
    disk_info: Vec<DiskInfo>,
    battery_info: Vec<BatteryInfo>,
}

impl SystemMonitor {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let mut sys = System::new_all();
        sys.refresh_all();

        let disks = Disks::new_with_refreshed_list();

        // Create process table
        let process_delgate = ProcessTableDelegate::new();
        let process_table = cx.new(|cx| {
            TableState::new(process_delgate, window, cx)
                .col_selectable(false)
                .col_movable(false)
        });

        let mut monitor = Self {
            sys,
            disks,
            data: VecDeque::with_capacity(MAX_DATA_POINTS),
            time_index: 0,
            active_tab: MonitorTab::System,
            process_table,
            disk_info: Vec::new(),
            battery_info: Vec::new(),
        };

        // Collect initial data
        monitor.collect_metrics(cx);

        // Start the update loop
        cx.spawn(async move |this, cx| {
            loop {
                Timer::after(INTERVAL).await;

                let result = this.update(cx, |this, cx| {
                    this.collect_metrics(cx);
                    cx.notify();
                });

                if result.is_err() {
                    break;
                }
            }
        })
        .detach();

        monitor
    }

    fn collect_metrics(&mut self, cx: &mut Context<'_, SystemMonitor>) {
        // Refresh system info
        self.sys.refresh_all();
        self.disks.refresh(true);

        // Calculate CPU usage
        let cpu_usage = self.sys.global_cpu_usage() as f64;

        // Calculate memeory usage
        let total_memory = self.sys.total_memory() as f64;
        let used_memory = self.sys.used_memory() as f64;
        let memory_usage = if total_memory > 0.0 {
            (used_memory / total_memory * 100.0).min(100.0)
        } else {
            0.0
        };

        // Create data point
        let point = MetricPoint {
            time: format!("{}s", self.time_index),
            cpu: cpu_usage,
            memory: memory_usage,
        };

        // Add to history
        if self.data.len() >= MAX_DATA_POINTS {
            self.data.pop_front();
        }
        self.data.push_back(point);
        self.time_index += 1;

        // Update process table
        self.process_table.update(cx, |table, cx| {
            table.delegate_mut().update_processes(&self.sys);
            cx.notify();
        });

        // Update disk info (take first disk for status bar)
        self.disk_info = self
            .disks
            .iter()
            .map(|disk| DiskInfo {
                name: disk.name().to_string_lossy().to_string(),
                total: disk.total_space(),
                used: disk.total_space() - disk.available_space(),
            })
            .collect();

        // Update battery info
        self.update_battery_info();
    }

    fn update_battery_info(&mut self) {
        self.battery_info.clear();

        if let Ok(manager) = battery::Manager::new()
            && let Ok(batteries) = manager.batteries()
        {
            for battery in batteries.flatten() {
                let soc = battery.state_of_charge().get::<percent>();

                let icon = match battery.state() {
                    battery::State::Charging => IconName::BatteryCharging,
                    battery::State::Discharging if soc < 20.0 => IconName::BatteryWarning,
                    battery::State::Discharging if soc < 80.0 => IconName::BatteryLow,
                    battery::State::Discharging => IconName::BatteryMedium,
                    battery::State::Empty => IconName::Battery,
                    battery::State::Full => IconName::BatteryFull,
                    _ => IconName::Battery,
                };

                self.battery_info.push(BatteryInfo {
                    model: battery
                        .model()
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| "Unknown Battery".to_string()),
                    icon,
                    percentage: soc,
                })
            }
        }
    }

    fn set_active_tab(&mut self, idx: usize, _window: &mut Window, cx: &mut Context<Self>) {
        self.active_tab = MonitorTab::from_index(idx);
        cx.notify();
    }

    fn render_chart(
        &self,
        title: &str,
        data: Vec<MetricPoint>,
        value_fn: impl Fn(&MetricPoint) -> f64 + 'static,
        color: Hsla,
        cx: &Context<Self>,
    ) -> impl IntoElement {
        v_flex()
            .min_h(px(160.))
            .flex_1()
            .gap_2()
            .border_1()
            .border_color(cx.theme().border)
            .child(
                h_flex().justify_between().py_1().px_3().child(
                    div()
                        .text_sm()
                        .text_color(cx.theme().foreground)
                        .child(title.to_string()),
                ),
            )
            .child(
                AreaChart::new(data)
                    .x(|d| d.time.clone())
                    .y(value_fn)
                    .stroke(color)
                    .fill(linear_gradient(
                        0.,
                        linear_color_stop(color.opacity(0.4), 1.),
                        linear_color_stop(cx.theme().background.opacity(0.1), 0.),
                    ))
                    .tick_margin(15),
            )
    }

    fn render_system_tab(&self, cx: &Context<Self>) -> impl IntoElement {
        let data: Vec<MetricPoint> = self.data.iter().cloned().collect();

        v_flex()
            .p_3()
            .gap_4()
            .flex_1()
            .child(self.render_chart("CPU Usage", data.clone(), |d| d.cpu, cx.theme().red, cx))
            .child(self.render_chart(
                "Memory Usage",
                data.clone(),
                |d| d.memory,
                cx.theme().blue,
                cx,
            ))
    }

    fn render_processes_tab(&self, _cx: &Context<Self>) -> impl IntoElement {
        v_flex().size_full().child(
            DataTable::new(&self.process_table)
                .bordered(false)
                .stripe(true)
                .small(),
        )
    }

    fn render_status_bar(&self, cx: &Context<Self>) -> impl IntoElement {
        let primary_dark = self.disk_info.first();
        let primary_battery = self.battery_info.first();

        h_flex()
            .px_3()
            .gap_4()
            .h_7()
            .text_sm()
            .items_center()
            .justify_between()
            .border_t_1()
            .border_color(cx.theme().border)
            .bg(cx.theme().tab_bar)
            .text_color(cx.theme().muted_foreground)
            .child(h_flex().gap_4().when_some(primary_dark, |this, disk| {
                let used_percent = if disk.total > 0 {
                    (disk.used as f64 / disk.total as f64 * 100.0) as f32
                } else {
                    0.0
                };
                this.child(h_flex())
            }))
    }
}

impl Render for SystemMonitor {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .flex()
            .justify_center()
            .items_center()
            .child("System Monitor")
    }
}

fn main() {
    application().run(|cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(960.), px(640.)), cx);
        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(bounds)),
            ..Default::default()
        };

        // cx.open_window(options, |_window, cx| cx.new(|_| SystemMonitor))
        //     .expect("failed to open system monitor window");
        cx.activate(true);
    });
}
