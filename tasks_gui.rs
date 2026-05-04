use gtk::prelude::*;

fn main() {
    // Initialize GTK
    gtk::premain();

    // Create a task manager
    let mut task_manager = TaskManager::new();
    task_manager.add_task("Task 1");
    task_manager.add_task("Task 2");

    // Get the tasks view widget
    let tasks_view = get_widget::<gtk::ScrolledWindow>("tasks_view");

    // Connect the "size-allocate" signal to a callback function
    tasks_view.connect_size_allocate(|_, size| {
        // Pack the radio buttons into the scrolled window
        for (index, task) in task_manager.tasks.iter().enumerate() {
            let radio_button = gtk::RadioButton::new(&task.name, index);
            tasks_view.pack_start(&radio_button, false, false, 0);
        }

        // Connect the "toggled" signal to a callback function
        tasks_view.connect_toggled(|_, _| {
            for (index, task) in task_manager.tasks.iter().enumerate() {
                let radio_button = task_manager.tasks.get(index).unwrap();
                if radio_button == task {
                    // Show the label when the task is selected
                    gtk::main_loop();
                    break;
                }
            }
        });

        // Connect the "delete" signal to a callback function
        tasks_view.connect_delete_event(|_, event| {
            for (index, task) in task_manager.tasks.iter().enumerate() {
                let radio_button = task_manager.tasks.get(index).unwrap();
                if radio_button == task {
                    // Remove the task from the task manager
                    task_manager.delete_task(index);
                    break;
                }
            }

            // Return true to propagate the event
            false
        });

        // Connect the "list" signal to a callback function
        tasks_view.connect_list_event(|_, _| {
            for (index, task) in task_manager.tasks.iter().enumerate() {
                let radio_button = task_manager.tasks.get(index).unwrap();
                if radio_button == task {
                    // Print the task name to the console
                    println!("{}", index);
                    break;
                }
            }

            // Return true to propagate the event
            false
        });

        // Connect the "set_status" signal to a callback function
        tasks_view.connect_set_status(|_, status| {
            for (index, task) in task_manager.tasks.iter().enumerate() {
                let radio_button = task_manager.tasks.get(index).unwrap();
                if radio_button == task {
                    // Print a message to the console
                    println!("Task status set.");
                    break;
                }
            }

            // Return true to propagate the event
            false
        });
    });

    gtk::main_loop();
}
