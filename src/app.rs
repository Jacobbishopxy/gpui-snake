//! file: app.rs
//! author: Jacob Xie
//! date: 2025/12/14 23:45:00 Sunday
//! brief:

use gpui::{
    App, AppContext, Application, Bounds, Focusable, KeyBinding, Timer, WindowBounds,
    WindowOptions, px, size,
};

use crate::game::{
    MoveDown, MoveLeft, MoveRight, MoveUp, QuitGame, RestartGame, SnakeGame, TogglePause,
};

pub fn run() {
    Application::new().run(|cx: &mut App| {
        cx.bind_keys([
            KeyBinding::new("up", MoveUp, None),
            KeyBinding::new("down", MoveDown, None),
            KeyBinding::new("left", MoveLeft, None),
            KeyBinding::new("right", MoveRight, None),
            KeyBinding::new("w", MoveUp, None),
            KeyBinding::new("s", MoveDown, None),
            KeyBinding::new("a", MoveLeft, None),
            KeyBinding::new("d", MoveRight, None),
            KeyBinding::new("space", TogglePause, None),
            KeyBinding::new("enter", RestartGame, None),
            KeyBinding::new("escape", QuitGame, None),
        ]);

        let bounds = Bounds::centered(None, size(px(880.), px(720.)), cx);
        let window = cx
            .open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    ..Default::default()
                },
                |_, cx| cx.new(SnakeGame::new),
            )
            .unwrap();

        let game = window
            .update(cx, |view: &mut SnakeGame, window, cx| {
                window.focus(&view.focus_handle(cx));
                cx.activate(true);
                cx.entity()
            })
            .unwrap();

        spawn_game_loop(game.clone(), cx);
        cx.on_action(|_: &QuitGame, cx| cx.quit());
        cx.activate(true);
    });
}

fn spawn_game_loop(game: gpui::Entity<SnakeGame>, cx: &mut App) {
    cx.spawn({
        async move |cx| loop {
            let delay = match game.read_with(cx, |game, _| game.tick_delay()) {
                Ok(duration) => duration,
                Err(_) => break,
            };

            Timer::after(delay).await;
            if game
                .update(cx, |game, cx| {
                    game.tick(cx);
                })
                .is_err()
            {
                break;
            }
        }
    })
    .detach();
}
