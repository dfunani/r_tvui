use crossterm::event;
use crossterm::event::Event;
use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use crossterm::event::KeyEventKind;
use ratatui::DefaultTerminal;
use ratatui::Frame;
use ratatui::style::Color;
use ratatui::style::Modifier;
use ratatui::style::Style;
use ratatui::widgets::Block;
use ratatui::widgets::Borders;
use ratatui::widgets::List;
use ratatui::widgets::ListItem;
use ratatui::widgets::ListState;
use std::cmp::Ordering;
use std::env::current_dir;
use std::fs::{DirEntry, read_dir};
use std::io::Result;
use std::path::PathBuf;
use std::time::Duration;
struct App {
    current_working_directory: PathBuf,
    artifacts: Vec<Artifact>,
    selection: usize,
    scroll_state: ListState,
}

enum AppState {
    ACTIVE,
    Quit,
}

impl App {
    pub fn new() -> Result<Self> {
        let current_working_directory = current_dir()?;
        let artifacts = get_directory_entries(&current_working_directory)?;
        let mut scroll_state = ListState::default();
        let selection = 0;
        let mut scroll_index = None;

        if !artifacts.is_empty() {
            scroll_index = Some(0);
        }
        scroll_state.select(scroll_index);

        Ok(Self {
            current_working_directory,
            artifacts,
            selection,
            scroll_state,
        })
    }

    pub fn reload(&mut self) -> Result<()> {
        let artifacts = get_directory_entries(&self.current_working_directory)?;

        if self.selection >= artifacts.len() {
            self.selection = artifacts.len().saturating_sub(1);
        }

        if artifacts.is_empty() {
            self.scroll_state.select(None);
        } else {
            self.scroll_state.select(Some(self.selection));
        };
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
enum ArtifactType {
    Directory,
    File,
    Symlink,
    Other,
}

#[derive(Debug, Clone)]
struct Artifact {
    name: String,
    artifact_type: ArtifactType,
}

fn main() -> Result<()> {
    global_exception_handler();
    let _ = ratatui::run(app_loop);

    Ok(())
}
fn app_loop(terminal: &mut DefaultTerminal) -> Result<()> {
    // 1. read_dir current directory into Vec<(name, is_dir)>
    let mut app = App::new()?;

    // 2. loop: draw list with selected index highlighted
    loop {
        terminal.draw(|frame| draw(frame, &mut app))?;

        match handle_key_events(&mut app)? {
            AppState::Quit => break,
            _ => {}
        }
    }
    // loop {
    // }
    // 3. poll key: j/k move, l enter dir, h parent, q break
    Ok(())
}

fn global_exception_handler() {
    let original = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        ratatui::restore();
        original(info);
    }));
}

fn get_directory_entries(path: &PathBuf) -> Result<Vec<Artifact>> {
    let mut entries = Vec::new();

    for entry in read_dir(path)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().to_string();

        let artifact_type = get_artifact_type(&entry)?;

        entries.push(Artifact {
            name,
            artifact_type,
        });
    }

    entries.sort_by(sort_directory_function);
    Ok(entries)
}

fn get_artifact_type(entry: &DirEntry) -> Result<ArtifactType> {
    let entry_type = entry.file_type()?;

    if entry_type.is_dir() {
        return Ok(ArtifactType::Directory);
    }
    if entry_type.is_file() {
        return Ok(ArtifactType::File);
    }
    if entry_type.is_symlink() {
        return Ok(ArtifactType::Symlink);
    }
    Ok(ArtifactType::Other)
}

fn sort_directory_function<'a, 'b>(a: &'a Artifact, b: &'b Artifact) -> Ordering {
    match (&a.artifact_type, &b.artifact_type) {
        (ArtifactType::Directory, ArtifactType::File) => Ordering::Less,
        (ArtifactType::File, ArtifactType::Directory) => Ordering::Greater,
        _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
    }
}

fn get_title_block<'a>(title: &'a String) -> Block<'a> {
    let title = format!(" {}", title);
    Block::default().title(title).borders(Borders::ALL)
}

fn get_list_block<'a>(entries: &'a Vec<ListItem>, block: &'a Block) -> List<'a> {
    List::new(entries.clone())
        .block(block.clone())
        .highlight_style(
            Style::new()
                .add_modifier(Modifier::REVERSED)
                .fg(Color::Cyan),
        )
}

fn draw(frame: &mut Frame, app: &mut App) {
    let entries: Vec<ListItem> = app
        .artifacts
        .iter()
        .map(|entry| {
            let label = &entry.artifact_type;
            match label {
                ArtifactType::Directory => format!("{}/", entry.name),
                _ => entry.name.clone(),
            }
        })
        .map(ListItem::from)
        .collect::<Vec<ListItem>>();

    let title = app.current_working_directory.display().to_string();
    let block = get_title_block(&title);
    let list = get_list_block(&entries, &block);
    frame.render_stateful_widget(list, frame.area(), &mut app.scroll_state)
}

fn handle_key_events(app: &mut App) -> Result<AppState> {
    let mut check_event_status = event::poll(Duration::from_millis(250))?;
    if !check_event_status {
        return Ok(AppState::ACTIVE);
    }

    if check_event_status {
        let Event::Key(key) = event::read()? else {
            return Ok(AppState::ACTIVE);
        };

        if key.kind != KeyEventKind::Press {
            return Ok(AppState::ACTIVE);
        }

        match key.code {
            KeyCode::Char('q') => {
                return Ok(AppState::Quit);
            }
            KeyCode::Esc => {
                return Ok(AppState::Quit);
            }
            _ => {}
        }
        handle_key_event(app, key)?;
    }
    Ok(AppState::ACTIVE)
}

fn handle_key_event(app: &mut App, event_key: KeyEvent) -> Result<()> {
    match event_key.code {
        KeyCode::Char('w') | KeyCode::Up => {
            if app.selection > 0 {
                app.selection -= 1;
                app.scroll_state.select(Some(app.selection));
            }
        }
        KeyCode::Char('s') | KeyCode::Down => {
            if app.selection + 1 < app.artifacts.len() {
                app.selection += 1;
                app.scroll_state.select(Some(app.selection));
            }
        }
        KeyCode::Char('a') | KeyCode::Left => {
            if app.current_working_directory.pop() {
                app.selection = 0;
                app.reload()?;
            }
        }
        KeyCode::Char('d') | KeyCode::Right | KeyCode::Enter => {
            let Some(selection) = app.scroll_state.selected() else {
                return Ok(());
            };
            let Some(artifact) = app.artifacts.get(selection) else {
                return Ok(());
            };
            if artifact.artifact_type == ArtifactType::Directory {
                app.current_working_directory.push(&artifact.name);
                app.selection = 0;
                app.reload()?;
            }
        }
        _ => {}
    }
    Ok(())
}
