pub struct OnStartupAction {
    /// TODO: return an error instead of unwrap.
    callback: Box<dyn Fn() -> Result<(), String>>,
}

#[allow(dead_code)]
impl OnStartupAction {
    pub fn new(callback: impl Fn() -> Result<(), String> + 'static) -> Self {
        OnStartupAction { 
            callback: Box::new(callback),
        }
    }
}

impl OnStartupAction {
    #[inline]
    pub fn exec(&self) -> Result<(), String> {
        (self.callback)()
    }
}
