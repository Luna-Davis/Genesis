mod create;
mod delete;

use std::env;

pub enum Task {
    Create,
    Delete,
}

pub fn task_selector(task: Task, name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let current_dir = env::current_dir()?.to_path_buf();
    let file_name = current_dir
        .join(&name)
        .to_str()
        .ok_or("Invalid UTF-8 in path")?
        .to_string();
    match task {
        Task::Create => {
            create::creator(&file_name)?;
            Ok(())
        }
        Task::Delete => {
            delete::deleter(&file_name)?;
            Ok(())
        }
    }
}
