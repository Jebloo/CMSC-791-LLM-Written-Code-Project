fn main() {
    // Create a new instance of the task manager
    let mut task_manager = TaskManager::new();

    loop {
        println!("1. Add task");
        println!("2. Delete task");
        println!("3. List tasks");
        println!("4. Set task status");
        println!("5. Exit");

        let mut input = String::new();
        std::io::stdin().read_line(&mut input).expect("Failed to read line");
        let choice: usize = match input.trim().parse() {
            Ok(num) => num,
            Err(_) => continue,
        };

        match choice {
            1 => task_manager.add_task("Task 1"),
            2 => {
                if task_manager.tasks.is_empty() {
                    println!("No tasks to delete.");
                } else {
                    let mut input = String::new();
                    std::io::stdin().read_line(&mut input).expect("Failed to read line");
                    let index: usize = match input.trim().parse() {
                        Ok(num) => num,
                        Err(_) => continue,
                    };

                    if 0 <= index && index < task_manager.tasks.len() {
                        task_manager.delete_task(index);
                    } else {
                        eprintln!("Task index out of bounds.");
                    }
                }
            },
            3 => task_manager.list_tasks(),
            4 => {
                let mut input = String::new();
                std::io::stdin().read_line(&mut input).expect("Failed to read line");
                let name: &str = input.trim();
                if !name.is_empty() {
                    let status: &str = "Running";
                    task_manager.set_task_status(name, status);
                }
            },
            5 => break,
            _ => println!("Invalid choice. Please try again."),
        }
    }

    // Save the tasks to a file
    std::fs::write("tasks.txt", serde_json::to_string(&task_manager.tasks).unwrap()).expect("Failed to write data");
}
