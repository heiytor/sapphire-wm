use super::on_startup::OnStartupAction;

#[allow(dead_code)]
impl OnStartupAction {
    pub fn spawn(process: &str) -> Box<dyn Fn() -> Result<(), String>> {
        let process_parts: Vec<&str> = process.split_whitespace().collect();

        match process_parts.split_first() {
            Some((command, args)) => {
                let command = command.to_string();
                let args: Vec<String> = args.iter().map(|&s| s.to_string()).collect();

                Box::new(move || {
                    std::process::Command::new(&command)
                        .args(&args)
                        .spawn()
                        .map_err(|e| e.to_string())?;

                    Ok(())
                })
            },
            None => {
                Box::new(move || Err("Invalid process string".to_string()))
            },
        }
    }
}
