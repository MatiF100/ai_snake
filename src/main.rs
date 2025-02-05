use rand::prelude::*;
use raylib::prelude::*;

// The following snake game implementation
// Is based on official raylib example
// Original code at: https://github.com/raysan5/raylib-games/blob/master/classics/src/snake.c
const SNAKE_LEN: usize = 256;
const SQUARE_SIZE: isize = 31;

#[derive(Default, Copy, Clone)]
struct Snake {
    size: Vector2,
    speed: Vector2,
    color: Color,
}

#[derive(Default)]
struct Food {
    size: Vector2,
    active: bool,
    color: Color,
}

enum Move {
    FW,
    BW,
    LT,
    RT,
    PS,
}
enum Mode {
    // Window and Raylib stuff
    Keyboard,
    // Interfacing with external controls
    External {
        moves: std::sync::mpsc::Receiver<Move>,
        reward: std::sync::mpsc::Sender<isize>,
    },
}

struct WindowData<'a> {
    handle: &'a mut RaylibHandle,
    thread: &'a mut RaylibThread,
    frames_counter: usize,

    allow_move: bool,
    offset: Vector2,
    pause: bool,
}

struct GameState<'a> {
    control_mode: Mode,
    window: Option<WindowData<'a>>,

    // Representation of game (environment)
    // Shouldn't rely on Raylib
    game_over: bool,

    fruit_position: Option<(isize, isize)>,
    snake_position: Vec<(isize, isize)>,
    snake_velocity: (isize, isize),
    counter_tail: isize,
    board_size: (isize, isize),
}

impl<'a> GameState<'a> {
    fn init(with_window: Option<(&'a mut RaylibHandle, &'a mut RaylibThread)>) -> Self {
        let window = if let Some((h, t)) = with_window {
            Some(WindowData {
                handle: h,
                thread: t,
                frames_counter: 0,
                allow_move: false,
                offset: Vector2 { x: 0.0, y: 0.0 },
                pause: false,
            })
        } else {
            None
        };

        Self {
            window,
            control_mode: Mode::Keyboard,

            game_over: false,

            counter_tail: 1,

            snake_position: vec![(0, 0)],
            fruit_position: None,
            snake_velocity: (1, 0),
            board_size: (16, 16),
        }
    }
    fn reset(&mut self) {
        self.game_over = false;
        self.counter_tail = 1;
        self.snake_position = vec![(0, 0)];
        self.fruit_position = None;
        self.snake_velocity = (1, 0);
    }
    fn update_game(&mut self) {
        match self.control_mode {
            Mode::Keyboard => {
                if let Some(window) = &mut self.window {
                    if !self.game_over {
                        if !window.pause {
                            if window.allow_move {
                                match window.handle.get_key_pressed() {
                                    Some(KeyboardKey::KEY_RIGHT) => {
                                        if self.snake_velocity.0 == 0 {
                                            self.snake_velocity = (1, 0);
                                        }
                                        window.allow_move = false;
                                    }
                                    Some(KeyboardKey::KEY_LEFT) => {
                                        if self.snake_velocity.0 == 0 {
                                            self.snake_velocity = (-1, 0);
                                        }
                                        window.allow_move = false;
                                    }
                                    Some(KeyboardKey::KEY_DOWN) => {
                                        if self.snake_velocity.1 == 0 {
                                            self.snake_velocity = (0, 1);
                                        }
                                        window.allow_move = false;
                                    }
                                    Some(KeyboardKey::KEY_UP) => {
                                        if self.snake_velocity.1 == 0 {
                                            self.snake_velocity = (0, -1);
                                        }
                                        window.allow_move = false;
                                    }
                                    _ => (),
                                }
                            }

                            let last_position = self.snake_position.last().unwrap().clone();
                            if window.frames_counter % 10 == 0 {
                                let saved_position = self.snake_position[0];
                                for i in (1..self.snake_position.len()).rev() {
                                    self.snake_position[i] = self.snake_position[i - 1];
                                }
                                self.snake_position[0].0 = saved_position.0 + self.snake_velocity.0;
                                self.snake_position[0].1 = saved_position.1 + self.snake_velocity.1;

                                window.allow_move = true;
                            }
                            window.frames_counter += 1;

                            if self.snake_position[0].0 >= self.board_size.0
                                || self.snake_position[0].1 >= self.board_size.1
                                || self.snake_position[0].0 < 0
                                || self.snake_position[0].1 < 0
                            {
                                self.game_over = true;
                            }

                            for i in 1..self.snake_position.len() {
                                if self.snake_position[0] == self.snake_position[i] {
                                    self.game_over = true;
                                }
                            }

                            if let None = self.fruit_position {
                                let mut rng = rand::rng();
                                let random_position = (
                                    rng.random_range(0..self.board_size.0 as i64) as isize,
                                    rng.random_range(0..self.board_size.1 as i64) as isize,
                                );

                                if self.snake_position.iter().any(|p| *p == random_position) {
                                    self.fruit_position = Some(last_position);
                                } else {
                                    self.fruit_position = Some(random_position);
                                }
                            };

                            if let Some(pos) = self.fruit_position {
                                if self.snake_position[0] == pos {
                                    self.snake_position.push(last_position);
                                    self.fruit_position = None;
                                }
                            }
                        }
                    } else if window.handle.is_key_pressed(KeyboardKey::KEY_ENTER) {
                        self.reset();
                    }
                }
            }
            _ => (),
        }
    }
    fn draw_game(&mut self) {
        match &mut self.window {
            Some(window) => {
                let mut context = window.handle.begin_drawing(&window.thread);

                context.clear_background(Color::RAYWHITE);
                if !self.game_over {
                    //Grid lines
                    for i in 0..=self.board_size.0 {
                        context.draw_line_v(
                            Vector2 {
                                x: (SQUARE_SIZE * i) as f32 + window.offset.x / 2.0,
                                y: window.offset.y / 2.0,
                            },
                            Vector2 {
                                x: (SQUARE_SIZE * i) as f32 + window.offset.x / 2.0,
                                y: (self.board_size.1 * SQUARE_SIZE) as f32 - window.offset.y / 2.0,
                            },
                            Color::LIGHTGRAY,
                        )
                    }

                    for i in 0..=self.board_size.1 {
                        context.draw_line_v(
                            Vector2 {
                                x: window.offset.x / 2.0,
                                y: (SQUARE_SIZE * i) as f32 + window.offset.y / 2.0,
                            },
                            Vector2 {
                                x: (self.board_size.0 * SQUARE_SIZE) as f32 - window.offset.x / 2.0,
                                y: (SQUARE_SIZE * i) as f32 + window.offset.y / 2.0,
                            },
                            Color::LIGHTGRAY,
                        )
                    }

                    //Snake
                    for (idx, snake_segment) in self.snake_position.iter().enumerate() {
                        context.draw_rectangle_v(
                            Vector2 {
                                x: (snake_segment.0 * SQUARE_SIZE) as f32,
                                y: (snake_segment.1 * SQUARE_SIZE) as f32,
                            },
                            Vector2 {
                                x: SQUARE_SIZE as f32,
                                y: SQUARE_SIZE as f32,
                            },
                            if idx == 0 {
                                Color::DARKBLUE
                            } else {
                                Color::BLUE
                            },
                        );
                    }

                    //Fruit
                    if let Some(pos) = self.fruit_position {
                        context.draw_rectangle_v(
                            Vector2 {
                                x: (pos.0 * SQUARE_SIZE) as f32,
                                y: (pos.1 * SQUARE_SIZE) as f32,
                            },
                            Vector2 {
                                x: SQUARE_SIZE as f32,
                                y: SQUARE_SIZE as f32,
                            },
                            Color::GREEN,
                        );
                    }

                    //Pause screen

                    if window.pause {
                        context.draw_text(
                            "GAME PAUSED",
                            ((self.board_size.0 * SQUARE_SIZE) / 2
                                - context.measure_text("GAME PAUSED", 40) as isize)
                                as i32,
                            ((self.board_size.1 * SQUARE_SIZE) / 2) as i32 - 40,
                            40,
                            Color::GRAY,
                        );
                    }
                } else {
                    let msg = "PRESS [ENTER] TO PLAY AGAIN";
                    context.draw_text(
                        msg,
                        ((self.board_size.0 * SQUARE_SIZE) / 2) as i32
                            - context.measure_text(msg, 20) / 2,
                        ((self.board_size.1 * SQUARE_SIZE) / 2 - 40) as i32,
                        40,
                        Color::GRAY,
                    );
                }
            }
            _ => return,
        };
    }

    fn run_as_game(&mut self) {
        loop {
            if let Some(window) = &mut self.window {
                if window.handle.window_should_close() {
                    return;
                }
            }
            self.update_game();
            self.draw_game();
        }
    }
}

fn main() {
    let (mut rl, mut thread) = raylib::init()
        .size(800, 600)
        .resizable()
        //.undecorated()
        .title("Test")
        .build();

    rl.set_target_fps(60);
    let mut game_state = GameState::init(Some((&mut rl, &mut thread)));
    game_state.run_as_game();

    println!("Hello, world!");
}
