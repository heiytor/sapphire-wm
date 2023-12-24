use crate::event_context::EventContext;

pub struct OnKeypressAction {
    cb: Box<dyn Fn(EventContext) -> Result<(), String>>,

    pub modkey: u16,
    pub ch: char,
}

#[allow(dead_code)]
impl OnKeypressAction {
    pub fn new(modkey: u16, ch: char, handler: impl Fn(EventContext) -> Result<(), String> + 'static) -> Self {
        OnKeypressAction { 
            modkey,
            ch,
            cb: Box::new(handler),
        }
    }
    
    pub fn spawn_process(modkey: u16, ch: char, process: String) -> Self {
        OnKeypressAction {
            modkey,
            ch,
            cb: Box::new(move |_| {
                std::process::Command::new(&process).spawn().map_err(|e| e.to_string())?;
                Ok(())
            }),
        }
    }

    pub fn kill_process(modkey: u16, ch: char) -> Self {
        OnKeypressAction {
            modkey,
            ch,
            cb: Box::new(move |ctx| {
                xcb::destroy_window(&ctx.conn, ctx.get_active_window()?);
                Ok(())
            }),
        }
    }
}

impl OnKeypressAction {
    #[inline]
    pub fn exec(&self, ctx: EventContext) -> Result<(), String> {
        (self.cb)(ctx)
    }
}

