use crate::errors::Error;

pub trait FnOnStartup: dyn_clone::DynClone {
    fn call(&self) -> Result<(), Error>;
}

impl<F> FnOnStartup for F
where 
    F: Fn() -> Result<(), Error> + Clone 
{
    fn call(&self) -> Result<(), Error> {
        self()
    }
}

pub struct OnStartup {
    callback: Box<dyn FnOnStartup>,
}

#[allow(dead_code)]
impl OnStartup {
    pub fn new(callback: Box<dyn FnOnStartup>) -> Self {
        OnStartup {
            callback,
        }
    }
}

impl Clone for OnStartup {
    fn clone(&self) -> Self {
        Self {
            callback: dyn_clone::clone_box(&*self.callback),
        }
    }
}

impl OnStartup {
    #[inline]
    pub fn call(&self) -> Result<(), Error> {
        self.callback.call()
    }
}
