use crate::app::{BrowserAction, ActionDispatcher};
use super::NavigationModel;


pub struct NavigationModelImpl {
    dispatcher: Box<dyn ActionDispatcher>
}

impl NavigationModelImpl {

    pub fn new(dispatcher: Box<dyn ActionDispatcher>) -> Self {
        Self { dispatcher }
    }
}

impl NavigationModel for NavigationModelImpl {

    fn go_back(&self) {
        self.dispatcher.dispatch(BrowserAction::NavigationPop.into())
    }

    fn can_go_back(&self) -> bool {
        true
    }
}