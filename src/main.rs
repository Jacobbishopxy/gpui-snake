use std::{
    collections::{HashSet, VecDeque},
    time::Duration,
};

use gpui::{
    App, Application, Bounds, Context, FocusHandle, Focusable, KeyBinding, Render, Timer, Window,
    WindowBounds, WindowOptions, actions, div, prelude::*, px, rgb, rgba, size,
};
use rand::{Rng, SeedableRng, rngs::StdRng};

const GRID_WIDTH: i32 = 24;
const GRID_HEIGHT: i32 = 20;
const CELL_SIZE: f32 = 26.0;
const BASE_TICK_MS: u64 = 150;
const MIN_TICK_MS: u64 = 70;
const SPEED_STEP_MS: u64 = 4;

actions!(
    snake,
    [
        MoveUp,
        MoveDown,
        MoveLeft,
        MoveRight,
        TogglePause,
        RestartGame,
        QuitGame
    ]
);

fn main() {
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
            .update(cx, |view, window, cx| {
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

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct Cell {
    x: i32,
    y: i32,
}

impl Cell {
    fn offset(self, direction: Direction) -> Self {
        let (dx, dy) = direction.vector();
        Self {
            x: self.x + dx,
            y: self.y + dy,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    fn vector(self) -> (i32, i32) {
        match self {
            Direction::Up => (0, -1),
            Direction::Down => (0, 1),
            Direction::Left => (-1, 0),
            Direction::Right => (1, 0),
        }
    }

    fn is_opposite(self, other: Direction) -> bool {
        matches!(
            (self, other),
            (Direction::Up, Direction::Down)
                | (Direction::Down, Direction::Up)
                | (Direction::Left, Direction::Right)
                | (Direction::Right, Direction::Left)
        )
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum GameStatus {
    Ready,
    Running,
    Paused,
    GameOver,
}

struct SnakeGame {
    board_width: i32,
    board_height: i32,
    snake: VecDeque<Cell>,
    direction: Direction,
    next_direction: Direction,
    food: Cell,
    rng: StdRng,
    state: GameStatus,
    score: u32,
    high_score: u32,
    focus_handle: FocusHandle,
    base_tick_ms: u64,
    min_tick_ms: u64,
    cell_px: f32,
}

impl SnakeGame {
    fn new(cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();
        let mut rng = StdRng::from_entropy();
        let board_width = GRID_WIDTH;
        let board_height = GRID_HEIGHT;
        let snake = Self::build_initial_snake(board_width, board_height);
        let food = Self::random_food(&snake, &mut rng, board_width, board_height);

        Self {
            board_width,
            board_height,
            snake,
            direction: Direction::Right,
            next_direction: Direction::Right,
            food,
            rng,
            state: GameStatus::Ready,
            score: 0,
            high_score: 0,
            focus_handle,
            base_tick_ms: BASE_TICK_MS,
            min_tick_ms: MIN_TICK_MS,
            cell_px: CELL_SIZE,
        }
    }

    fn build_initial_snake(width: i32, height: i32) -> VecDeque<Cell> {
        let mut body = VecDeque::new();
        let start_x = width / 2;
        let start_y = height / 2;
        for offset in 0..4 {
            body.push_back(Cell {
                x: start_x - offset,
                y: start_y,
            });
        }
        body
    }

    fn random_food(snake: &VecDeque<Cell>, rng: &mut StdRng, width: i32, height: i32) -> Cell {
        loop {
            let cell = Cell {
                x: rng.gen_range(0..width),
                y: rng.gen_range(0..height),
            };
            if !snake.contains(&cell) {
                return cell;
            }
        }
    }

    fn tick_delay(&self) -> Duration {
        let speedup = (self.score / 4) as u64 * SPEED_STEP_MS;
        let ms = self
            .base_tick_ms
            .saturating_sub(speedup)
            .max(self.min_tick_ms);
        Duration::from_millis(ms)
    }

    fn board_contains(&self, cell: &Cell) -> bool {
        (0..self.board_width).contains(&cell.x) && (0..self.board_height).contains(&cell.y)
    }

    fn queue_direction(&mut self, direction: Direction) {
        if matches!(self.state, GameStatus::GameOver | GameStatus::Ready) {
            return;
        }
        if direction.is_opposite(self.direction) && self.snake.len() > 1 {
            return;
        }
        self.next_direction = direction;
    }

    fn toggle_pause(&mut self) {
        self.state = match self.state {
            GameStatus::Running => GameStatus::Paused,
            GameStatus::Paused => GameStatus::Running,
            other => other,
        };
    }

    fn reset(&mut self) {
        self.snake = Self::build_initial_snake(self.board_width, self.board_height);
        self.direction = Direction::Right;
        self.next_direction = Direction::Right;
        self.state = GameStatus::Ready;
        self.score = 0;
        self.food = self.random_empty_cell();
    }

    fn random_empty_cell(&mut self) -> Cell {
        Self::random_food(
            &self.snake,
            &mut self.rng,
            self.board_width,
            self.board_height,
        )
    }

    fn handle_turn(&mut self, direction: Direction, cx: &mut Context<Self>) {
        self.queue_direction(direction);
        cx.notify();
    }

    fn handle_restart(&mut self, cx: &mut Context<Self>) {
        match self.state {
            GameStatus::Ready => self.state = GameStatus::Running,
            GameStatus::Running => {
                self.reset();
                self.state = GameStatus::Running;
            }
            GameStatus::Paused => self.state = GameStatus::Running,
            GameStatus::GameOver => {
                self.reset();
                self.state = GameStatus::Running;
            }
        }
        cx.notify();
    }

    fn handle_toggle_pause(&mut self, cx: &mut Context<Self>) {
        if matches!(self.state, GameStatus::Running | GameStatus::Paused) {
            self.toggle_pause();
            cx.notify();
        }
    }

    fn status_text(&self) -> (&'static str, u32) {
        match self.state {
            GameStatus::Ready => ("Ready", 0x93c5fd),
            GameStatus::Running => ("Running", 0x34d399),
            GameStatus::Paused => ("Paused", 0xfbbf24),
            GameStatus::GameOver => ("Game Over", 0xf87171),
        }
    }

    fn tick(&mut self, cx: &mut Context<Self>) {
        if self.state != GameStatus::Running {
            return;
        }
        if let Some(head) = self.snake.front().copied() {
            self.direction = self.next_direction;
            let next = head.offset(self.direction);

            if !self.board_contains(&next) || self.snake.contains(&next) {
                self.state = GameStatus::GameOver;
                cx.notify();
                return;
            }

            self.snake.push_front(next);
            if next == self.food {
                self.score += 1;
                self.high_score = self.high_score.max(self.score);
                self.food = self.random_empty_cell();
            } else {
                self.snake.pop_back();
            }
            cx.notify();
        }
    }
}

impl Render for SnakeGame {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let (status_text, status_color) = self.status_text();
        let is_focused = self.focus_handle(cx).is_focused(window);

        let snake_lookup: HashSet<Cell> = self.snake.iter().copied().collect();
        let head = self.snake.front().copied();
        let cell_size = px(self.cell_px);

        let grid = div()
            .flex()
            .flex_col()
            .gap_1()
            .children((0..self.board_height).map(|y| {
                div()
                    .flex()
                    .gap_1()
                    .children((0..self.board_width).map(|x| {
                        let cell = Cell { x, y };
                        let color = if Some(cell) == head {
                            rgb(0x34d399)
                        } else if cell == self.food {
                            rgb(0xf97316)
                        } else if snake_lookup.contains(&cell) {
                            rgb(0x10b981)
                        } else {
                            rgb(0x0f172a)
                        };

                        div().w(cell_size).h(cell_size).rounded_sm().bg(color)
                    }))
            }));

        let instructions = [
            "Enter to start or restart",
            "Arrows / WASD to steer",
            "Space to pause or resume",
            "Esc to quit",
        ];

        div()
            .bg(rgb(0x020617))
            .text_color(rgb(0xf8fafc))
            .size_full()
            .p_5()
            .gap_4()
            .flex()
            .flex_col()
            .track_focus(&self.focus_handle(cx))
            .key_context("gpui-snake")
            .on_action(cx.listener(|this, _: &MoveUp, _, cx| this.handle_turn(Direction::Up, cx)))
            .on_action(
                cx.listener(|this, _: &MoveDown, _, cx| this.handle_turn(Direction::Down, cx)),
            )
            .on_action(
                cx.listener(|this, _: &MoveLeft, _, cx| this.handle_turn(Direction::Left, cx)),
            )
            .on_action(
                cx.listener(|this, _: &MoveRight, _, cx| this.handle_turn(Direction::Right, cx)),
            )
            .on_action(cx.listener(|this, _: &RestartGame, _, cx| this.handle_restart(cx)))
            .on_action(cx.listener(|this, _: &TogglePause, _, cx| this.handle_toggle_pause(cx)))
            .child(
                div()
                    .flex()
                    .gap_4()
                    .items_center()
                    .child(div().text_3xl().child(format!("Score: {}", self.score)))
                    .child(
                        div()
                            .text_xl()
                            .text_color(rgb(0xa5f3fc))
                            .child(format!("Best: {}", self.high_score)),
                    )
                    .child(
                        div()
                            .text_lg()
                            .text_color(rgb(status_color))
                            .child(status_text),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgb(0x94a3b8))
                            .child(if is_focused {
                                "Focused"
                            } else {
                                "Click inside the window to take control"
                            }),
                    )
                    .child(
                        div()
                            .text_sm()
                            .child(format!("Tick: {}ms", self.tick_delay().as_millis())),
                    ),
            )
            .child({
                let overlay_text = match self.state {
                    GameStatus::Ready => Some("Press Enter to start"),
                    GameStatus::Paused => Some("Paused"),
                    GameStatus::GameOver => Some("Game Over – press Enter"),
                    GameStatus::Running => None,
                };

                div()
                    .p_4()
                    .rounded_2xl()
                    .bg(rgb(0x111827))
                    .shadow_lg()
                    .relative()
                    .child(grid)
                    .when_some(overlay_text, |this, message| {
                        this.child(
                            div()
                                .absolute()
                                .top(px(0.))
                                .bottom(px(0.))
                                .left(px(0.))
                                .right(px(0.))
                                .flex()
                                .items_center()
                                .justify_center()
                                .bg(rgba(0x020617A6))
                                .text_xl()
                                .text_color(rgb(0xf8fafc))
                                .child(message),
                        )
                    })
            })
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .gap_3()
                    .text_sm()
                    .text_color(rgb(0xcbd5f5))
                    .children(instructions.into_iter().map(|text| {
                        div()
                            .px_3()
                            .py_2()
                            .rounded_md()
                            .bg(rgb(0x1e293b))
                            .child(text)
                    })),
            )
    }
}

impl Focusable for SnakeGame {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}
