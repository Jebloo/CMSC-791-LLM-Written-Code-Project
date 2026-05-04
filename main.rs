use gtk4 as gtk;
use gtk::prelude::*;
use sysinfo::{System, SystemExt, ProcessExt, CpuExt};
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::VecDeque;

const HISTORY_SIZE: usize = 60; // 60 seconds of history

struct SystemState {
    sys: System,
    cpu_history: VecDeque<f32>,
    ram_history: VecDeque<f32>,
}

fn main() {
    let app = gtk::Application::builder().application_id("com.task.manager").build();
    app.connect_activate(build_ui);
    app.run();
}

fn build_ui(app: &gtk::Application) {
    let window = gtk::ApplicationWindow::builder()
        .application(app)
        .title("Advanced Task Manager")
        .default_width(800)
        .default_height(600)
        .build();

    let main_box = gtk::Box::new(gtk::Orientation::Vertical, 10);
    
    // --- GRAPH SECTION ---
    let graph_area = gtk::DrawingArea::new();
    graph_area.set_content_height(200);
    
    // --- PROCESS LIST SECTION ---
    let scrolled = gtk::ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .vexpand(true)
        .build();
    let process_list = gtk::ListBox::new();
    scrolled.set_child(Some(&process_list));

    main_box.append(&gtk::Label::new(Some("System Performance (60s)")));
    main_box.append(&graph_area);
    main_box.append(&gtk::Label::new(Some("Top Processes (by RAM)")));
    main_box.append(&scrolled);

    window.set_child(Some(&main_box));

    // Shared State
    let state = Rc::new(RefCell::new(SystemState {
        sys: System::new_all(),
        cpu_history: VecDeque::from(vec![0.0; HISTORY_SIZE]),
        ram_history: VecDeque::from(vec![0.0; HISTORY_SIZE]),
    }));

    // 1. Drawing Logic (The Graph)
    let state_draw = state.clone();
    graph_area.set_draw_func(move |_, cr, width, height| {
        let s = state_draw.borrow();
        draw_chart(cr, &s.cpu_history, width as f64, height as f64, (0.0, 0.6, 0.0)); // Green CPU
        draw_chart(cr, &s.ram_history, width as f64, height as f64, (0.0, 0.4, 0.9)); // Blue RAM
    });

    // 2. Update Loop (Every 1 second)
    let state_update = state.clone();
    glib::timeout_add_local(std::time::Duration::from_millis(1000), move || {
        let mut s = state_update.borrow_mut();
        s.sys.refresh_all();

        // Update History
        let cpu = s.sys.global_cpu_info().cpu_usage() / 100.0;
        let ram = s.sys.used_memory() as f32 / s.sys.total_memory() as f32;
        
        s.cpu_history.pop_front();
        s.cpu_history.push_back(cpu);
        s.ram_history.pop_front();
        s.ram_history.push_back(ram);

        // Update Process List
        process_list.remove_all();
        let mut procs: Vec<_> = s.sys.processes().values().collect();
        procs.sort_by(|a, b| b.memory().cmp(&a.memory())); // Sort by RAM

        for proc in procs.iter().take(10) {
            let row = gtk::Box::new(gtk::Orientation::Horizontal, 10);
            row.append(&gtk::Label::builder().label(&proc.pid().to_string()).width_request(50).build());
            row.append(&gtk::Label::builder().label(proc.name()).hexpand(true).halign(gtk::Align::Start).build());
            row.append(&gtk::Label::new(Some(&format!("{:.1} MB", proc.memory() / 1024 / 1024))));
            process_list.append(&row);
        }

        graph_area.queue_draw(); // Redraw graph
        glib::ControlFlow::Continue
    });

    window.show();
}

fn draw_chart(cr: &cairo::Context, data: &VecDeque<f32>, w: f64, h: f64, color: (f64, f64, f64)) {
    cr.set_source_rgb(color.0, color.1, color.2);
    cr.set_line_width(2.0);
    
    let step = w / (HISTORY_SIZE as f64 - 1.0);
    for (i, &val) in data.iter().enumerate() {
        let x = i as f64 * step;
        let y = h - (val as f64 * h);
        if i == 0 { cr.move_to(x, y); } else { cr.line_to(x, y); }
    }
    cr.stroke().expect("Draw failed");
}
