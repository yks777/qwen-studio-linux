use std::time::Instant;

pub struct UpdateManager {
    last_check: Option<Instant>,
    cached_info: Option<crate::config::schema::UpdateInfo>,
}

impl UpdateManager {
    pub fn new() -> Self {
        Self {
            last_check: None,
            cached_info: None,
        }
    }

    pub fn set_last_check(&mut self) {
        self.last_check = Some(Instant::now());
    }

    pub fn cache_info(&mut self, info: crate::config::schema::UpdateInfo) {
        self.cached_info = Some(info);
    }

    pub fn get_cached(&self) -> Option<&crate::config::schema::UpdateInfo> {
        self.cached_info.as_ref()
    }
}
