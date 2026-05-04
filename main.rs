use gtk4 as gtk;
use gtk::prelude::*;
use gtk::gio;
use sysinfo::System;
use std::rc::Rc;
use std::cell::RefCell;

fn main() {
    let app = gtk::Application::builder()
        .application_id("com.example.taskmanager")
        .build();
    app.connect_activate(build_ui);
    app.run();
}

// Helper to convert bytes to human-readable format
fn format_memory(bytes: u64) -> String {
    let mb = bytes / 1024 / 1024;
    if mb > 1024 {
        format!("{:.2} GB", mb as f64 / 1024.0)
    } else {
        format!("{} MB", mb)
    }
}

fn build_ui(app: &gtk::Application) {
    let window = gtk::ApplicationWindow::builder()
        .application(app)
        .title("Rust Task Manager Pro")
        .default_width(900)
        .default_height(700)
        .build();

    let notebook = gtk::Notebook::new();
    let sys = Rc::new(RefCell::new(System::new_all()));

    // --- TAB 1: PROCESSES ---
    let process_box = gtk::Box::new(gtk::Orientation::Vertical, 5);
    let scrolled_window = gtk::ScrolledWindow::builder()
        .vexpand(true)
        .build();

    let list_store = gio::ListStore::new::<ProcessWrapper>();
    let selection_model = gtk::SingleSelection::new(Some(list_store.clone()));
    let column_view = gtk::ColumnView::new(Some(selection_model));

    // Name Column
    let name_factory = gtk::SignalListItemFactory::new();
    name_factory.connect_setup(|_, li| {
        li.set_child(Some(&gtk::Label::builder().halign(gtk::Align::Start).margin_start(10).build()));
    });
    name_factory.connect_bind(|_, li| {
        let w = li.item().unwrap().downcast::<ProcessWrapper>().unwrap();
        let label = li.child().unwrap().downcast::<gtk::Label>().unwrap();
        label.set_text(&w.name());
    });
    column_view.append_column(&gtk::ColumnViewColumn::new(Some("Name"), Some(name_factory)));

    // PID Column
    let pid_factory = gtk::SignalListItemFactory::new();
    pid_factory.connect_setup(|_, li| li.set_child(Some(&gtk::Label::new(None))));
    pid_factory.connect_bind(|_, li| {
        let w = li.item().unwrap().downcast::<ProcessWrapper>().unwrap();
        let label = li.child().unwrap().downcast::<gtk::Label>().unwrap();
        label.set_text(&w.pid().to_string());
    });
    column_view.append_column(&gtk::ColumnViewColumn::new(Some("PID"), Some(pid_factory)));

    // Virtual Memory Column
    let vmem_factory = gtk::SignalListItemFactory::new();
    vmem_factory.connect_setup(|_, li| li.set_child(Some(&gtk::Label::builder().margin_end(10).build())));
    vmem_factory.connect_bind(|_, li| {
        let w = li.item().unwrap().downcast::<ProcessWrapper>().unwrap();
        let label = li.child().unwrap().downcast::<gtk::Label>().unwrap();
        label.set_text(&format_memory(w.vmem()));
    });
    column_view.append_column(&gtk::ColumnViewColumn::new(Some("Virtual Memory"), Some(vmem_factory)));

    scrolled_window.set_child(Some(&column_view));
    process_box.append(&scrolled_window);
    notebook.append_page(&process_box, Some(&gtk::Label::new(Some("Processes"))));

    // --- TAB 2: PERFORMANCE (With Graph & Labels) ---
    // FIXED: Explicit margins instead of margin_all
    let perf_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .margin_top(20)
        .margin_bottom(20)
        .margin_start(20)
        .margin_end(20)
        .spacing(10)
        .build();

    let graph_area = gtk::DrawingArea::builder()
        .vexpand(true)
        .build();

    let history = Rc::new(RefCell::new(vec![0.0f32; 60])); 
    let history_clone = history.clone();

    graph_area.set_draw_func(move |_, cr, w, h| {
        let width = w as f64;
        let height = h as f64;
        let padding = 45.0; // Space for Y-axis labels
        let graph_w = width - padding - 10.0;
        let graph_h = height - 40.0; // Space for X-axis labels

        // Background
        cr.set_source_rgb(0.01, 0.01, 0.01);
        cr.paint().unwrap();

        // Draw Axes Labels
        cr.set_source_rgb(0.8, 0.8, 0.8);
        cr.set_font_size(11.0);
        
        for i in 0..=4 {
            let val = 100 - (i * 25);
            let y = 10.0 + (graph_h / 4.0) * i as f64;
            
            // Y-axis text
            cr.move_to(5.0, y + 5.0);
            cr.show_text(&format!("{}%", val)).unwrap();
            
            // Horizontal Grid Lines
            cr.set_source_rgba(0.0, 1.0, 0.0, 0.15);
            cr.move_to(padding, y);
            cr.line_to(width, y);
            cr.stroke().unwrap();
            cr.set_source_rgb(0.8, 0.8, 0.8);
        }

        // Draw the Graph
        let data = history_clone.borrow();
        cr.set_source_rgb(0.0, 0.9, 0.0);
        cr.set_line_width(2.0);
        let step = graph_w / 59.0;
        
        let first_y = (10.0 + graph_h) - (data[0] as f64 / 100.0 * graph_h);
        cr.move_to(padding, first_y);
        
        for (i, &val) in data.iter().enumerate() {
            let x = padding + (i as f64 * step);
            let y = (10.0 + graph_h) - (val as f64 / 100.0 * graph_h);
            cr.line_to(x, y);
        }
        cr.stroke().unwrap();

        // X-axis Label
        cr.move_to(padding + (graph_w / 2.0) - 30.0, height - 5.0);
        cr.show_text("Last 60 Seconds").unwrap();
    });

    perf_box.append(&gtk::Label::new(Some("CPU Performance History")));
    perf_box.append(&graph_area);
    notebook.append_page(&perf_box, Some(&gtk::Label::new(Some("Performance"))));

    // --- REFRESH LOGIC ---
    let sys_ref = sys.clone();
    let history_ref = history.clone();
    let area_ref = graph_area.clone();
    let store_ref = list_store.clone();

    glib::timeout_add_local(std::time::Duration::from_millis(1000), move || {
        let mut s = sys_ref.borrow_mut();
        s.refresh_all();

        // Update Graph
        let mut h = history_ref.borrow_mut();
        h.remove(0);
        h.push(s.global_cpu_info().cpu_usage());
        area_ref.queue_draw();

        // Update Process List (Top 40 by Virtual Memory)
        store_ref.remove_all();
        let mut procs: Vec<_> = s.processes().values().collect();
        procs.sort_by(|a, b| b.virtual_memory().cmp(&a.virtual_memory()));
        
        for p in procs.iter().take(40) {
            store_ref.append(&ProcessWrapper::new(p.name().to_string(), p.pid().as_u32(), p.virtual_memory()));
        }

        glib::ControlFlow::Continue
    });

    window.set_child(Some(&notebook));
    window.show();
}

// --- GOBJECT WRAPPER ---
mod imp {
    use gtk4 as gtk;
    use gtk::glib;
    use gtk::subclass::prelude::*;
    use std::cell::RefCell;

    #[derive(Default)]
    pub struct ProcessWrapper {
        pub name: RefCell<String>,
        pub pid: RefCell<u32>,
        pub vmem: RefCell<u64>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ProcessWrapper {
        const NAME: &'static str = "ProcessWrapper";
        type Type = super::ProcessWrapper;
    }
    impl ObjectImpl for ProcessWrapper {}
}

glib::wrapper! {
    pub struct ProcessWrapper(ObjectSubclass<imp::ProcessWrapper>);
}

use glib::subclass::types::ObjectSubclassIsExt;

impl ProcessWrapper {
    pub fn new(name: String, pid: u32, vmem: u64) -> Self {
        let obj: Self = glib::Object::new();
        obj.imp().name.replace(name);
        obj.imp().pid.replace(pid);
        obj.imp().vmem.replace(vmem);
        obj
    }
    pub fn name(&self) -> String { self.imp().name.borrow().clone() }
    pub fn pid(&self) -> u32 { *self.imp().pid.borrow() }
    pub fn vmem(&self) -> u64 { *self.imp().vmem.borrow() }
}
