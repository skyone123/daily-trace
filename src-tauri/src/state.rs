use crate::capture::Collector;
use crate::store::Store;
use std::sync::Arc;

pub struct AppState {
    pub store: Arc<Store>,
    pub collector: Arc<Collector>,
}

impl AppState {
    pub fn new(store: Arc<Store>, collector: Arc<Collector>) -> Self {
        AppState { store, collector }
    }
}
