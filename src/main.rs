use std::{
    error::Error,
    io::{self, Stdout},
    sync::mpsc::{self, channel},
    thread,
    time::Duration,
};

use crossterm::{
    cursor::{Hide, Show},
    event,
    event::{Event, KeyCode},
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use invaders::{
    frame::{self, Drawable},
    player::Player,
    render,
};

fn main() -> Result<(), Box<dyn Error>> {
    // Terminal
    let mut stdout = io::stdout();
    terminal::enable_raw_mode()?;
    stdout.execute(EnterAlternateScreen)?;
    stdout.execute(Hide)?;

    // Render loop in a separated thread
    let (render_tx, render_rx) = mpsc::channel();

    let render_handle = thread::spawn(move || {
        let mut last_frame = frame::new_frame();
        let mut stdout = io::stdout();

        render::render(&mut stdout, &last_frame, &last_frame, true);

        loop {
            let curr_frame = match render_rx.recv() {
                Ok(x) => x,
                Err(_) => break,
            };

            render::render(&mut stdout, &last_frame, &curr_frame, false);
            last_frame = curr_frame;
        }
    });

    // Game Loop
    let mut player = Player::new();

    'gameloop: loop {
        // Per-frame init
        let mut curr_frame = frame::new_frame();

        // Input
        while event::poll(Duration::default())? {
            if let Event::Key(key_event) = event::read()? {
                match key_event.code {
                    KeyCode::Left | KeyCode::Char('a') => player.move_left(),
                    KeyCode::Right | KeyCode::Char('d') => player.move_right(),
                    KeyCode::Esc | KeyCode::Char('q') => break 'gameloop,
                    _ => {}
                }
            }
        }

        // Draw & Render
        player.draw(&mut curr_frame);
        let _ = render_tx.send(curr_frame);
        thread::sleep(Duration::from_millis(1));
    }

    // Cleanup
    drop(render_tx);
    render_handle.join().unwrap();
    stdout.execute(Show)?;
    stdout.execute(LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;
    Ok(())
}
