use gtk::prelude::*;

// Define our task manager struct
struct TaskManager {
    tasks: Vec<Task>,
}

impl TaskManager {
    // Create a new instance of the task manager
    fn new() -> Self {
        Self { tasks: Vec::new() }
    }

    // Add a task to the task manager
    fn add_task(&mut self, name: &str) {
        let task = Task::new(name, "Running");
        self.tasks.push(task);
    }

    // Delete a task from the task manager
    fn delete_task(&mut self, index: usize) {
        if 0 <= index && index < self.tasks.len() {
            self.tasks.remove(index);
        } else {
            eprintln!("Task index out of bounds.");
        }
    }

    // List all tasks in the task manager
    fn list_tasks(&self) {
        for (index, task) in self.tasks.iter().enumerate() {
            println!("{}: {}", index, task.name);
        }
    }

    // Set the status of a task
    fn set_task_status(&mut self, name: &str, status: &str) {
        let mut found = false;
        for (index, task) in self.tasks.iter().enumerate() {
            if task.name == name.to_string() {
                task.set_status(status);
                found = true;
                break;
            }
        }

        if !found {
            eprintln!("Task not found.");
        }
    }
}

struct Task {
    name: String,
    status: String,
}

impl Task {
    fn new(name: &str, status: &str) -> Self {
        Self { name: name.to_string(), status: status.to_string() }
    }

    fn set_status(&mut self, status: &str) {
        self.status = status.to_string();
    }
}
