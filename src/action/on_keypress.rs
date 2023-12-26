use crate::event_context::EventContext;

pub struct OnKeypressAction {
    callback: Box<dyn Fn(EventContext) -> Result<(), String>>,

    pub modkey: u16,
    pub ch: char,
}

impl OnKeypressAction {
    pub fn new(
        modkey: u16,
        ch: char,
        callback: impl Fn(EventContext) -> Result<(), String> + 'static,
    ) -> Self {
        OnKeypressAction { 
            modkey,
            ch,
            callback: Box::new(callback),
        }
    }
}

impl OnKeypressAction {
    #[inline]
    pub fn exec(&self, ctx: EventContext) -> Result<(), String> {
        (self.callback)(ctx)
    }
}

