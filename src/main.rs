use std::{thread, time::Duration};

use porcino_core::network::LayerSettings;
use rand::{prelude::*, random};
use raylib::prelude::*;

// The following snake game implementation
// Is based on official raylib example
// Original code at: https://github.com/raysan5/raylib-games/blob/master/classics/src/snake.c
const SQUARE_SIZE: isize = 31;

enum Move {
    TOP,
    BTM,
    LFT,
    RHT,
    PAS,
}

struct State {
    board: Vec<isize>,
    score: isize,
}
enum Mode {
    // Window and Raylib stuff
    Keyboard,
    // Interfacing with external controls
    External {
        move_queue: std::sync::mpsc::Receiver<Move>,
        state_queue: std::sync::mpsc::Sender<State>,
    },
}

struct WindowData<'a> {
    handle: &'a mut RaylibHandle,
    thread: &'a mut RaylibThread,
    frames_counter: usize,

    offset: Vector2,
    pause: bool,
}

struct GameState<'a> {
    control_mode: Mode,
    window: Option<WindowData<'a>>,

    // Representation of game (environment)
    // Shouldn't rely on Raylib
    game_over: bool,
    allow_move: bool,

    fruit_position: Option<(isize, isize)>,
    snake_position: Vec<(isize, isize)>,
    snake_velocity: (isize, isize),
    board_size: (isize, isize),

    score: isize,
}

impl<'a> GameState<'a> {
    fn create_threaded() -> (
        std::sync::mpsc::Sender<Move>,
        std::sync::mpsc::Receiver<State>,
    ) {
        let moves = std::sync::mpsc::channel::<Move>();
        let states = std::sync::mpsc::channel::<State>();

        std::thread::spawn(move || {
            let mut game = Self::init(None);
            game.control_mode = Mode::External {
                move_queue: moves.1,
                state_queue: states.0,
            };
            game.run_as_game();
        });

        (moves.0, states.1)
    }
    fn init(with_window: Option<(&'a mut RaylibHandle, &'a mut RaylibThread)>) -> Self {
        let window = if let Some((h, t)) = with_window {
            Some(WindowData {
                handle: h,
                thread: t,
                frames_counter: 0,
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
            allow_move: false,

            snake_position: vec![(8, 8)],
            fruit_position: None,
            snake_velocity: (1, 0),
            board_size: (16, 16),

            score: 0,
        }
    }
    fn reset(&mut self) {
        self.game_over = false;
        self.snake_position = vec![(8, 8)];
        self.fruit_position = None;
        self.score = 0;
        self.snake_velocity = (1, 0);
    }

    fn update_snake(&mut self) -> (isize, isize) {
        let last_position = self.snake_position.last().unwrap().clone();
        let saved_position = self.snake_position[0];
        for i in (1..self.snake_position.len()).rev() {
            self.snake_position[i] = self.snake_position[i - 1];
        }
        self.snake_position[0].0 = saved_position.0 + self.snake_velocity.0;
        self.snake_position[0].1 = saved_position.1 + self.snake_velocity.1;

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

        self.allow_move = false;
        self.score -= 1;
        last_position
    }

    fn update_env(&mut self, last_position: (isize, isize)) {
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
                self.score += 20;
                self.snake_position.push(last_position);
                self.fruit_position = None;
            }
        }
    }

    fn update_game(&mut self) {
        match &mut self.control_mode {
            Mode::Keyboard => {
                if let Some(window) = &mut self.window {
                    if !self.game_over {
                        if !window.pause {
                            match window.handle.get_key_pressed() {
                                Some(KeyboardKey::KEY_RIGHT) => {
                                    if self.snake_velocity.0 == 0 {
                                        self.snake_velocity = (1, 0);
                                        self.allow_move = false;
                                    }
                                }
                                Some(KeyboardKey::KEY_LEFT) => {
                                    if self.snake_velocity.0 == 0 {
                                        self.snake_velocity = (-1, 0);
                                        self.allow_move = false;
                                    }
                                }
                                Some(KeyboardKey::KEY_DOWN) => {
                                    if self.snake_velocity.1 == 0 {
                                        self.snake_velocity = (0, 1);
                                        self.allow_move = false;
                                    }
                                }
                                Some(KeyboardKey::KEY_UP) => {
                                    if self.snake_velocity.1 == 0 {
                                        self.snake_velocity = (0, -1);
                                        self.allow_move = false;
                                    }
                                }
                                _ => (),
                            }
                            if window.frames_counter % 10 == 0 {
                                self.allow_move = true;
                            }
                            window.frames_counter += 1;
                        }
                    } else if window.handle.is_key_pressed(KeyboardKey::KEY_ENTER) {
                        self.reset();
                    }
                } else {
                    panic!("Must use windowed mode for Keyboard control mode");
                }
            }
            Mode::External {
                move_queue,
                state_queue,
            } => {
                self.allow_move = true;
                if !self.game_over {
                } else {
                    self.reset();
                    return;
                }

                let mut external_state =
                    vec![0; self.board_size.0 as usize * self.board_size.1 as usize];
                for snake_part in &self.snake_position {
                    external_state[self.board_size.0 as usize * snake_part.0 as usize
                        + snake_part.1 as usize] = 1;
                }
                if let Some(pos) = &self.fruit_position {
                    external_state[pos.0 as usize * self.board_size.0 as usize + pos.1 as usize] =
                        2;
                }

                state_queue
                    .send(State {
                        board: external_state,
                        score: self.score,
                    })
                    .unwrap();

                match move_queue.recv().unwrap() {
                    Move::TOP => {
                        if self.snake_velocity.1 == 0 {
                            self.snake_velocity = (0, -1);
                        }
                    }
                    Move::BTM => {
                        if self.snake_velocity.1 == 0 {
                            self.snake_velocity = (0, 1);
                        }
                    }
                    Move::LFT => {
                        if self.snake_velocity.0 == 0 {
                            self.snake_velocity = (-1, 0);
                        }
                    }
                    Move::RHT => {
                        if self.snake_velocity.0 == 0 {
                            self.snake_velocity = (1, 0);
                        }
                    }
                    Move::PAS => {}
                }
            }
        }
        if self.allow_move {
            let last_valid_position = self.update_snake();
            self.update_env(last_valid_position);
        } else {
            self.update_env((0, 0));
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

                    //Score
                    context.draw_text(
                        &format!("Score: {}", self.score),
                        (self.board_size.0 as i32 + 1) * SQUARE_SIZE as i32,
                        SQUARE_SIZE as i32,
                        40,
                        Color::GRAY,
                    );

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

    let (send, rcv) = GameState::create_threaded();

    let mut game_state = GameState::init(Some((&mut rl, &mut thread)));
    let moves = std::sync::mpsc::channel::<Move>();
    let states = std::sync::mpsc::channel::<State>();

    game_state.control_mode = Mode::External {
        move_queue: moves.1,
        state_queue: states.0,
    };
    std::thread::spawn(move || {
        loop {
        let mut network = porcino_core::network::Network::new(
            vec![
                LayerSettings {
                    neurons: 16 * 16,
                    activation: porcino_core::network::Activations::Linear,
                },
                LayerSettings {
                    neurons: 150,
                    activation: porcino_core::network::Activations::Sigmoid,
                },
                LayerSettings {
                    neurons: 80,
                    activation: porcino_core::network::Activations::Sigmoid,
                },
                LayerSettings {
                    neurons: 5,
                    activation: porcino_core::network::Activations::Linear,
                },
            ],
            porcino_core::enums::InitializationMethods::Random,
        );
            let rc = states.1
                .recv()
                .unwrap()
                .board
                .iter()
                .map(|v| *v as f64)
                .collect::<Vec<_>>();

            let max_r = rc.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
            let rc = rc.iter().map(|v| v/max_r).collect::<Vec<_>>();
            network.process_data(&ndarray::Array2::from_shape_vec((rc.len(), 1), rc).unwrap());

            let output = <ndarray::ArrayBase<ndarray::OwnedRepr<f64>, ndarray::Dim<[usize; 2]>> as Clone>::clone(&network.layers.last().unwrap().state).into_raw_vec();
            let nn = output.iter().enumerate().max_by(|a, b| a.1.partial_cmp(b.1).unwrap()).unwrap().0;

            moves
                .0
                .send(match nn {
                    0 => Move::TOP,
                    1 => Move::BTM,
                    2 => Move::LFT,
                    3 => Move::RHT,
                    _ => Move::PAS,
                })
                .unwrap();
            thread::sleep(Duration::from_millis(10));
        }
    });
    game_state.run_as_game();

    println!("Hello, world!");
}
