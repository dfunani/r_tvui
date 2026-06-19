pub mod models {
    pub mod app;
    pub mod client;
    pub mod previewer;
}
pub mod ui;

pub mod events;

pub mod config {
    pub mod app;
    pub mod utils;
}

pub mod cli {
    pub mod model;
    pub mod utils;
}

pub mod os;

#[cfg(test)]
pub mod tests {
    pub mod app;
    pub mod cli;
    pub mod config;
    pub mod events;
    pub mod previewer;
    pub mod ui;
}
