use crate::event_context::EventContext;
use crate::action::on_keypress::OnKeypressAction;

#[allow(dead_code)]
impl OnKeypressAction {
    pub fn spawn(process: &str) -> Box<dyn Fn(EventContext) -> Result<(), String>> {
        let process = process.to_string();
        Box::new(move |_| {
            std::process::Command::new(&process).spawn().map_err(|e| e.to_string())?;
            Ok(())
        })
    }

    pub fn kill_process() -> Box<dyn Fn(EventContext) -> Result<(), String>> {
        Box::new(move |ctx| {
            xcb::destroy_window(&ctx.conn, ctx.get_active_window()?);
            Ok(())
        })
    }
}
