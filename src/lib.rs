pub mod models {
    pub mod app;
    pub mod async_app;
}
pub mod ui;

pub mod events;

pub mod config {
    pub mod app;
}

pub mod os;

#[cfg(test)]
pub mod tests {
    pub mod test_app;
}
