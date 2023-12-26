use super::on_startup::OnStartupAction;

#[allow(dead_code)]
impl OnStartupAction {
    pub fn spawn(process: &str) -> Box<dyn Fn() -> Result<(), String>> {
        let process = process.to_string();
        Box::new(move || {
            std::process::Command::new(&process).spawn().map_err(|e| e.to_string())?;
            Ok(())
        })
    }
}
