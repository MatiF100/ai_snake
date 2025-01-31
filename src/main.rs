use raylib::prelude::*;

// The following snake game implementation
// Is based on official raylib example
// Original code at: https://github.com/raysan5/raylib-games/blob/master/classics/src/snake.c
const SNAKE_LEN: usize = 256;
const SQUARE_SIZE: usize = 31;

#[derive(Default, Copy, Clone)]
struct Snake {
    position: Vector2,
    size: Vector2,
    speed: Vector2,
    color: Color,
}

#[derive(Default)]
struct Food {
    position: Vector2,
    size: Vector2,
    active: bool,
    color: Color,
}

struct GameState {
    screen_width: usize,
    screen_height: usize,

    frames_counter: usize,
    game_over: bool,
    pause: bool,

    fruit: Food,
    snake: [Snake; SNAKE_LEN],
    snake_position: [Vector2; SNAKE_LEN],

    allow_move: bool,
    offset: Vector2,
    counter_tail: usize,
}

impl GameState {
    fn init(screen_height: usize, screen_width: usize) -> Self {
        let mut snake_arr = [Snake::default(); SNAKE_LEN];

        for mut snake_part in &mut snake_arr {
            snake_part.position = Vector2 {
                x: (screen_width % SQUARE_SIZE / 2) as f32,
                y: (screen_height % SQUARE_SIZE / 2) as f32,
            };
            snake_part.size = Vector2 {
                x: SQUARE_SIZE as f32,
                y: SQUARE_SIZE as f32,
            };
            snake_part.speed = Vector2 {
                x: SQUARE_SIZE as f32,
                y: 0.0,
            };
            snake_part.color = Color::BLUE;
        }

        snake_arr[0].color = Color::DARKBLUE;

        Self {
            screen_height,
            screen_width,

            frames_counter: 0,
            game_over: false,
            pause: false,

            counter_tail: 1,
            allow_move: false,

            offset: Vector2 {
                x: (screen_width % SQUARE_SIZE) as f32,
                y: (screen_height % SQUARE_SIZE) as f32,
            },
            fruit: Food {
                size: Vector2 {
                    x: SQUARE_SIZE as f32,
                    y: SQUARE_SIZE as f32,
                },
                color: Color::SKYBLUE,
                active: false,
                ..Default::default()
            },

            snake: snake_arr,
            snake_position: [Vector2::zero(); SNAKE_LEN],
        }
    }
}

fn main() {
    let (mut rl, thread) = raylib::init()
        .size(800, 450)
        .resizable()
        //.undecorated()
        .title("Test")
        .build();

    let mut game_state = GameState::init(450, 400);

    rl.set_target_fps(60);

    while !rl.window_should_close() {
        update_game(&mut game_state, &mut rl);
        draw_game(&game_state, &mut rl, &thread);
    }
    println!("Hello, world!");
}

fn draw_game(state: &GameState, rl: &mut RaylibHandle, thread: &RaylibThread) {
    let mut context = rl.begin_drawing(thread);

    context.clear_background(Color::RAYWHITE);
    if !state.game_over {
        //Grid lines
        for i in 0..=state.screen_width / SQUARE_SIZE {
            context.draw_line_v(
                Vector2 {
                    x: (SQUARE_SIZE * i) as f32 + state.offset.x / 2.0,
                    y: state.offset.y / 2.0,
                },
                Vector2 {
                    x: (SQUARE_SIZE * i) as f32 + state.offset.x / 2.0,
                    y: state.screen_height as f32 - state.offset.y / 2.0,
                },
                Color::LIGHTGRAY,
            )
        }

        for i in 0..=state.screen_height / SQUARE_SIZE {
            context.draw_line_v(
                Vector2 {
                    x: state.offset.x / 2.0,
                    y: (SQUARE_SIZE * i) as f32 + state.offset.y / 2.0,
                },
                Vector2 {
                    x: state.screen_width as f32 - state.offset.x / 2.0,
                    y: (SQUARE_SIZE * i) as f32 + state.offset.y / 2.0,
                },
                Color::LIGHTGRAY,
            )
        }

        //Snake
        for i in 0..state.counter_tail {
            context.draw_rectangle_v(
                state.snake[i].position,
                state.snake[i].size,
                state.snake[i].color,
            );
        }

        //Fruit
        context.draw_rectangle_v(state.fruit.position, state.fruit.size, state.fruit.color);

        //Pause screen

        if state.pause {
            context.draw_text(
                "GAME PAUSED",
                (state.screen_width / 2 - context.measure_text("GAME PAUSED", 40) as usize) as i32,
                (state.screen_height / 2) as i32 - 40,
                40,
                Color::GRAY,
            );
        }
    } else {
        let msg = "PRESS [ENTER] TO PLAY AGAIN";
        context.draw_text(
            msg,
            (state.screen_width / 2) as i32 - context.measure_text(msg, 20) / 2,
            (state.screen_height / 2 - 40) as i32,
            40,
            Color::GRAY,
        );
    }
}

fn update_game(state: &mut GameState, rl: &mut RaylibHandle) {
    if !state.game_over {

        state.game_over = false;
        if !state.pause {
            if state.allow_move {
                match rl.get_key_pressed() {
                    Some(KeyboardKey::KEY_RIGHT) => {
                        if state.snake[0].speed.x == 0.0 {
                            state.snake[0].speed = Vector2 {
                                x: SQUARE_SIZE as f32,
                                y: 0.0,
                            };
                        }
                        state.allow_move = false;
                    }
                    Some(KeyboardKey::KEY_LEFT) => {
                        if state.snake[0].speed.x == 0.0 {
                            state.snake[0].speed = Vector2 {
                                x: -(SQUARE_SIZE as f32),
                                y: 0.0,
                            };
                        }
                        state.allow_move = false;
                    }
                    Some(KeyboardKey::KEY_DOWN) => {
                        if state.snake[0].speed.y == 0.0 {
                            state.snake[0].speed = Vector2 {
                                x: 0.0,
                                y: SQUARE_SIZE as f32,
                            };
                        }
                        state.allow_move = false;
                    }
                    Some(KeyboardKey::KEY_UP) => {
                        if state.snake[0].speed.y == 0.0 {
                            state.snake[0].speed = Vector2 {
                                x: 0.0,
                                y: -(SQUARE_SIZE as f32),
                            };
                        }
                        state.allow_move = false;
                    }
                    _ => (),
                }
            }

            for i in 0..state.counter_tail {
                state.snake_position[i] = state.snake[i].position;
            }

            if state.frames_counter % 10 == 0 {
                state.snake[0].position += state.snake[0].speed;
                state.allow_move = true;
                for i in 1..state.counter_tail {
                    state.snake[i].position = state.snake_position[i - 1];
                }
            }

            if state.snake[0].position.x > state.screen_width as f32 - state.offset.x
                || state.snake[0].position.y > state.screen_height as f32 - state.offset.y
                || state.snake[0].position.x < 0.0
                || state.snake[0].position.y < 0.0
            {
                state.game_over = true;
            }

            for i in 1..state.counter_tail {
                if state.snake[0].position == state.snake[i].position {
                    state.game_over = true;
                }
            }

            if !state.fruit.active {
                state.fruit.active = true;
                state.fruit.position = Vector2 {
                    x: (rl
                        .get_random_value::<f64>(0..(state.screen_width / SQUARE_SIZE - 1) as i32)
                        * SQUARE_SIZE as f64
                        + (state.offset.x / 2.0) as f64)
                        .trunc() as f32,
                    y: (rl
                        .get_random_value::<f64>(0..(state.screen_height / SQUARE_SIZE - 1) as i32)
                        * SQUARE_SIZE as f64
                        + (state.offset.y / 2.0) as f64)
                        .trunc() as f32,
                };

                for mut i in 0..state.counter_tail {
                    while state.fruit.position == state.snake[i].position {
                        state.fruit.position = Vector2 {
                            x: (rl.get_random_value::<f64>(
                                0..(state.screen_width / SQUARE_SIZE - 1) as i32,
                            ) * SQUARE_SIZE as f64
                                + (state.offset.x / 2.0) as f64)
                                .trunc() as f32,
                            y: (rl.get_random_value::<f64>(
                                0..(state.screen_height / SQUARE_SIZE - 1) as i32,
                            ) * SQUARE_SIZE as f64
                                + (state.offset.y / 2.0) as f64)
                                .trunc() as f32,
                        };
                        i = 0;
                    }
                }
            }

            if state.snake[0].position.x < state.fruit.position.x + state.fruit.size.x
                && state.snake[0].position.x + state.snake[0].size.x > state.fruit.position.x
                && state.snake[0].position.y < state.fruit.position.y + state.fruit.size.y
                && state.snake[0].position.y + state.snake[0].size.y > state.fruit.position.y
            {
                state.snake[state.counter_tail].position =
                    state.snake_position[state.counter_tail - 1];
                state.counter_tail += 1;
                state.fruit.active = false;
            }

            state.frames_counter += 1;
        }
    } else if rl.is_key_pressed(KeyboardKey::KEY_ENTER) {
        *state = GameState::init(
            rl.get_screen_height() as usize,
            (rl.get_screen_width() / 2) as usize,
        );
    }
}
