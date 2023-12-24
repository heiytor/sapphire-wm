pub struct OnStartupAction {
    /// TODO: return an error instead of unwrap.
    cb: Box<dyn Fn()>,
}

#[allow(dead_code)]
impl OnStartupAction {
    pub fn new(handler: impl Fn() + 'static) -> Self {
        OnStartupAction { 
            cb: Box::new(handler),
        }
    }
    
    pub fn spawn_process(process: String) -> Self {
        OnStartupAction {
            cb: Box::new(move || {
                std::process::Command::new(&process).spawn().unwrap();
            }),
        }
    }
}

impl OnStartupAction {
    #[inline]
    pub fn exec(&self) {
        (self.cb)()
    }
}
