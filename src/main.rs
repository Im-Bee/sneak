use std::{thread::sleep, time::Duration};

#[derive(Copy, Clone)]
struct Vec2i16 {
    x: i16,
    y: i16,
}

struct Square {
    position: Vec2i16,
    size:     Vec2i16,
}

const SPACE_CHAR:    u8 = ' '  as u8;
const NEW_LINE_CHAR: u8 = '\n' as u8;
const BOX_CHAR:      u8 = 178;
const W_KEY:         u8 = 87;
const A_KEY:         u8 = 65;
const D_KEY:         u8 = 68;
const S_KEY:         u8 = 83;
const Q_KEY:         u8 = 81;

#[cfg(windows)]
const STD_OUTPUT:     u32 = -11_i32 as u32;
#[cfg(windows)]
const WH_KEYBOARD_LL: i32 = 13;
#[cfg(windows)]
struct WinInput {
    hook_id: winapi::shared::windef::HHOOK,
}


#[cfg(windows)]
fn get_updated_term_vec2() -> Vec2i16 {
    use winapi::um::processenv::GetStdHandle;
    use winapi::um::wincon::GetConsoleScreenBufferInfo;
    use winapi::um::wincon::CONSOLE_SCREEN_BUFFER_INFO;
    use winapi::um::wincon::SMALL_RECT;
    use winapi::um::wincon::COORD;
    
    let mut csbi = CONSOLE_SCREEN_BUFFER_INFO {
        dwSize: COORD { X: (-1), Y: (-1) },
        dwCursorPosition: COORD { X: (-1), Y: (-1) },
        wAttributes: -1_i16 as u16,
        srWindow: SMALL_RECT { Left: (-1), Top: (-1), Right: (-1), Bottom: (-1) },
        dwMaximumWindowSize: COORD { X: (-1), Y: (-1) },
    };
    
    unsafe {
        let console_handle = GetStdHandle(STD_OUTPUT);
        
        GetConsoleScreenBufferInfo(console_handle, &mut csbi);
    }
    
    let dims = csbi.dwSize;
    Vec2i16 { x: dims.X, y: dims.Y }
}

#[cfg(windows)]
fn set_term_cursor_pos(dim: Vec2i16) {
    use winapi::um::processenv::GetStdHandle; 
    use winapi::um::wincon::SetConsoleCursorPosition;
    use winapi::um::wincon::COORD;

    unsafe {
        let console_handle = GetStdHandle(STD_OUTPUT);
        let coord = COORD { X: dim.x, Y: dim.y };
    
        SetConsoleCursorPosition(console_handle, coord);
    }
}

#[cfg(windows)]
fn output_sized_array(arr_ptr: *const u8, arr_size: i16) {
    use core::ptr::null_mut;
    use winapi::ctypes::c_void;
    use winapi::um::consoleapi::WriteConsoleA;
    use winapi::um::processenv::GetStdHandle;

    unsafe {
        let console_handle = GetStdHandle(STD_OUTPUT);

        WriteConsoleA(
            console_handle, 
            arr_ptr as *const c_void,
            arr_size as u32,
            null_mut(),
            null_mut());
    }
}

fn output_array(arr: &Vec<u8>) {
    output_sized_array(arr.as_ptr(), arr.len() as i16);
}

fn output_new_line() {
    static NEW_LINE_STR: [u8; 1] =  [ NEW_LINE_CHAR ];
    output_sized_array(NEW_LINE_STR.as_ptr(), NEW_LINE_STR.len() as i16);
}

fn output_string(msg : &str) {
    output_sized_array(msg.as_ptr(), msg.len() as i16);
    output_new_line();
}

#[cfg(windows)]
fn set_up_keyboard_hook_on_this_thread() -> WinInput {
    use winapi::um::winuser::SetWindowsHookExA;

    let mut r = WinInput { hook_id: std::ptr::null_mut() };

    unsafe {
        r.hook_id = SetWindowsHookExA(
            WH_KEYBOARD_LL, 
            Some(windows_ll_hook), 
            std::ptr::null_mut(), 
            0);
    }
    return r;
}

static mut KEY: u32 = 20;

#[cfg(windows)]
unsafe extern "system" fn windows_ll_hook(
    code: i32, 
    w_param: usize, 
    l_param: isize) -> isize {
    use winapi::um::winuser::CallNextHookEx;
    use winapi::um::winuser::KBDLLHOOKSTRUCT;
    use winapi::um::winuser::WM_KEYDOWN;
    
    let kbd: &KBDLLHOOKSTRUCT = (l_param as *const KBDLLHOOKSTRUCT).as_ref().unwrap();
    
    if w_param == WM_KEYDOWN as usize {
        KEY = kbd.vkCode;
    }

    CallNextHookEx(std::ptr::null_mut(), code, w_param, l_param)
}

#[cfg(windows)]
fn end_keyboard_hook_on_this_thread(wi: &mut WinInput) {
    use winapi::um::winuser::UnhookWindowsHookEx;

    unsafe {
        UnhookWindowsHookEx(wi.hook_id);
    }

    wi.hook_id = std::ptr::null_mut();
}

mod term_steady_out {
    use std::mem::swap;
    use std::usize;
    use crate::get_updated_term_vec2;
    use crate::set_term_cursor_pos;
    use crate::output_sized_array;
    use crate::Vec2i16;
    use crate::Square;
    use crate::BOX_CHAR;
    use crate::SPACE_CHAR;
    use private::SteadyRender;

    pub struct Renderer {
        terminal_dim: Vec2i16,
        back_buffer: Vec<u8>,
        front_buffer: Vec<u8>,
        objects: Vec<*const MashedPixels>,
    }

    pub struct MashedPixels {
        pub sqare: Square,
    }

    pub trait Render: private::SteadyRender {
        fn paint_whole_screen_in_letter_a(&mut self);
        fn render(&mut self);
    }

    mod private {
        pub trait SteadyRender {
            fn resize(&mut self);
            fn paint_whole_screen(&mut self);
            fn clear_whole_screen(&mut self);
            fn stamp_obj(&mut self, index: &usize);
            fn update_objs(&mut self);
            fn swap_buffers(&mut self);
            fn steady_render(&mut self);
        }
    }

    impl Renderer {
        // Initialize the renderer component,
        // get the current terminal dimensions, 
        // create two vectors for back and front buffer,
        // create a vector for objects that are assoscieted with the component
        pub fn initialize() -> Self {
            let mut r = Renderer { 
                terminal_dim: crate::get_updated_term_vec2(),
                back_buffer: Vec::<u8>::new(),
                front_buffer: Vec::<u8>::new(), 
                objects: Vec::<*const MashedPixels>::new(), 
            };

            r.clear_whole_screen();
            r.swap_buffers();
            r.clear_whole_screen();

            return r;
        }
    }

    impl MashedPixels {
        // Create new object in passed renderer
        // return mutable reference for the object,
        // set it as updated for it to be rendered
        pub fn initialize(& self, output: &mut Renderer) {
            let ptr = (self as *const MashedPixels).clone();
            output.objects.push(ptr);
        }

        pub fn set_pos(&mut self, pos: Vec2i16) {
            self.sqare.position = pos;
        }

        pub fn get_pos(&self) -> &Vec2i16 {
            &self.sqare.position
        }

        pub fn get_size(&self) -> &Vec2i16 {
            &self.sqare.size
        }
    }

    impl Render for Renderer {
        fn render(&mut self) {
            self.resize();
            self.clear_whole_screen();
            self.update_objs();
            self.swap_buffers();
            self.steady_render();
        }

        fn paint_whole_screen_in_letter_a(&mut self) {
            self.paint_whole_screen();
        }
    }

    impl private::SteadyRender for Renderer  {
        fn resize(&mut self) {
            self.terminal_dim = get_updated_term_vec2();
            let d = (self.terminal_dim.x * self.terminal_dim.y) as usize;

            if (d != self.back_buffer.len()) || (d != self.front_buffer.len()) {
                self.back_buffer.resize(d, SPACE_CHAR);
                self.front_buffer.resize(d, 0);
                self.clear_whole_screen();
                self.swap_buffers();
                self.clear_whole_screen();
                self.paint_whole_screen();
            }
        }

        fn paint_whole_screen(&mut self) {
            let d = &self.terminal_dim;
            
            set_term_cursor_pos(Vec2i16 { x: 0, y: 0 });
            crate::output_sized_array(
                self.front_buffer.as_ptr(), 
                self.front_buffer.len() as i16 - d.x);
        }
        
        fn clear_whole_screen(&mut self) {
            self.back_buffer.fill(SPACE_CHAR);
        }

        fn stamp_obj(&mut self, index: &usize) {
            let pixels = &mut unsafe { self.objects[*index].as_ref().unwrap() };
    
            for i in pixels.sqare.position.y
                ..(pixels.sqare.position.y + pixels.sqare.size.y) {
                let c_y = i as i32 * self.terminal_dim.x as i32;

                for k in pixels.sqare.position.x
                    ..(pixels.sqare.position.x + pixels.sqare.size.x) {
                    if (c_y + k as i32) as usize >= self.back_buffer.len() &&
                        (c_y + k as i32) < 0 {
                         return;
                    }
                    self.back_buffer[(c_y + k as i32) as usize] = BOX_CHAR;
                }
            }
        }

        fn update_objs(&mut self) {
            for i in 0..self.objects.len() {
                self.stamp_obj(&i);
            }
        }

        fn swap_buffers(&mut self) {
            swap(&mut self.back_buffer, &mut self.front_buffer);
        }

        fn steady_render(&mut self) {
            let d = &self.terminal_dim;
            
            set_term_cursor_pos(Vec2i16 { x: 0, y: 0 });

            let mut anchor_point: usize = usize::MAX;
            for i in 0..self.front_buffer.len() {
                if  (anchor_point == usize::MAX) && (self.back_buffer[i] != self.front_buffer[i]) {
                        anchor_point = i;
                }
                if (anchor_point != usize::MAX) && (self.back_buffer[i] == self.front_buffer[i]) {
                        set_term_cursor_pos(Vec2i16{ x: anchor_point as i16 % d.x, y: anchor_point as i16 / d.x });
                        output_sized_array(&self.front_buffer[anchor_point], (i - anchor_point) as i16);
                        set_term_cursor_pos(Vec2i16{ x: 0, y: 0 });

                        anchor_point = usize::MAX;
                }
            }
    
            if anchor_point != usize::MAX {
                 crate::output_sized_array(&self.front_buffer[anchor_point], (self.front_buffer.len() - 1 - anchor_point) as i16);
            }
        }
    }
}

mod game_logic {
    use std::sync::atomic::Ordering;
    use std::usize;
    use crate::{A_KEY, D_KEY, Q_KEY, S_KEY, W_KEY};
    use crate::{get_updated_term_vec2, term_steady_out::{MashedPixels, Renderer}, Vec2i16, Square};
    
    pub struct Game {
        pub alive: bool,
        world: World,
        tick: u64,
        main_actor: Sneak,
        sneak_peaces: Vec<Peace>,
        apples: Vec<Apple>,
        last_input: std::sync::Arc<std::sync::atomic::AtomicU32>
    }

    struct World {
        size: Vec2i16,
        center: Vec2i16,
        walls:  Vec<MashedPixels>,
    }

    enum Direction {
        Up,
        Right,
        Down,
        Left,
    }

    struct Apple {
        pixels: Box<MashedPixels>,
        alive: bool,
    }

    struct Peace {
        pixels: Box<MashedPixels>,
        index: usize,
    }
    
    struct Sneak {
        pixels: Box<MashedPixels>,
        direction: Direction,
        collected: i32,
    }

    impl World {
        pub fn initialize(output: &mut Renderer) -> Self {
            let mut r = World {
                size: get_updated_term_vec2(),
                center: get_updated_term_vec2(),
                walls:  Vec::<MashedPixels>::new(),
            };
            r.size.x = r.size.x - 1;
            r.size.y = r.size.y - 1;

            r.center.y = r.center.y - 1;
            let term_dims = &r.center;
            r.walls.push(MashedPixels {
                sqare: Square { position: (Vec2i16 { x: 0, y: 0 }), 
                                size:     (Vec2i16 { x: 1, y: term_dims.y }) }
            });
            r.walls.push(MashedPixels {
                sqare: Square { position: (Vec2i16 { x: term_dims.x - 1, y: 0 }), 
                                size:     (Vec2i16 { x: 1, y: term_dims.y }) }
            });
            r.walls.push(MashedPixels {
                sqare: Square { position: (Vec2i16 { x: 0, y: 0 }), 
                                size:     (Vec2i16 { x: term_dims.x, y: 1 }) }
            });
            r.walls.push(MashedPixels {
                sqare: Square { position: (Vec2i16 { x: 0, y: term_dims.y - 1 }), 
                                size:     (Vec2i16 { x: term_dims.x, y: 1 }) }
            });
            
            for i in &r.walls {
                i.initialize(output);
            }

            r.center.x = r.center.x / 2;
            r.center.y = r.center.y / 2;

            return r;
        }
    }

    impl Sneak {
        pub fn initialize(output: &mut Renderer, world: &World) -> Self {
            let r = Sneak {
                pixels: Box::new(MashedPixels {
                    sqare: Square { position: (Vec2i16 { 
                                                    x: world.center.x,
                                                    y: world.center.y }), 
                                    size:     (Vec2i16 { x: 1, y: 1 }) },
                }),
                direction: Direction::Up,
                collected: 0
            };
            
            r.pixels.initialize(output);

            return r;
        }
    }

    impl Game {
        pub fn initialize(output: &mut Renderer, input: &crate::term_input::Input) -> Self {
            let w = World::initialize(output);
            let ma = Sneak::initialize(output, &w);
            let mut apples_vec = Vec::<Apple>::new();
            let mut sneak_vec = Vec::<Peace>::new();
            for _i in 0..12 {
                apples_vec.push(Apple {
                    pixels: Box::new(MashedPixels {
                        sqare: Square { position: (Vec2i16 { 
                            x: -1,
                            y: -1 }), 
                            size:     (Vec2i16 { x: 1, y: 1 }) },
                    }),
                    alive: false,
                } );

                apples_vec.last().unwrap().pixels.initialize(output);
            }

            for _i in 0..(w.size.y * w.size.x) {
                sneak_vec.push(Peace {
                    pixels: Box::new(MashedPixels {
                        sqare: Square { position: (Vec2i16 { 
                            x: -1,
                            y: -1 }), 
                            size:     (Vec2i16 { x: 1, y: 1 }) },
                    }),
                    index: -1_isize as usize,
                } );

                sneak_vec.last().unwrap().pixels.initialize(output);
            }

            Game {
                alive: true,
                world: w,
                tick: 0,
                main_actor: ma,
                apples: apples_vec,
                sneak_peaces: sneak_vec,
                last_input: input.last.clone(),
            }
        }
        
        pub fn get_score(&mut self) -> i32 {
            self.main_actor.collected
        }

        pub fn update(&mut self) {
            let pos = self.main_actor.pixels.get_pos();
            let last_pos: Vec2i16 = Vec2i16 {
                x: pos.x,
                y: pos.y,
            };
            
            let last_keystroke = self.last_input.load(Ordering::Relaxed);
            if last_keystroke == W_KEY as u32 {
                self.main_actor.direction = Direction::Up;
            }
            if last_keystroke == D_KEY as u32 {
                self.main_actor.direction = Direction::Right;
            }
            if last_keystroke == S_KEY as u32 {
                self.main_actor.direction = Direction::Down;
            }
            if last_keystroke == A_KEY as u32 {
                self.main_actor.direction = Direction::Left;
            }
            if last_keystroke == Q_KEY as u32 {
                self.alive = false;
            }

            match self.main_actor.direction {
                Direction::Up => {
                    self.main_actor.pixels.set_pos(Vec2i16 {
                        x: pos.x,
                        y: pos.y - 1 });
                }
                Direction::Right => {
                    self.main_actor.pixels.set_pos(Vec2i16 { 
                        x: pos.x + 1,
                        y: pos.y });
                }
                Direction::Down => {
                    self.main_actor.pixels.set_pos(Vec2i16 { 
                        x: pos.x, 
                        y: pos.y + 1 });
                }
                Direction::Left => {
                    self.main_actor.pixels.set_pos(Vec2i16 { 
                        x: pos.x - 1, 
                        y: pos.y });
                }
            }
        
            let cur_snake_pos = self.main_actor.pixels.get_pos().clone();
            if self.tick % 20 == 0 {
                let mut random_vec = crate::game_logic::Game::random_vec2i16();

                random_vec.y = (random_vec.y + 1) % self.world.size.y;    
                random_vec.x = (random_vec.x + 1) % self.world.size.x;    

                for i in self.apples.iter_mut() {
                    if i.alive {
                        continue;
                    }

                    i.pixels.set_pos(random_vec);
                    i.alive = true;
                    break;
                }
            }
            if crate::game_logic::Game::check_is_in_deadly_collison(&self, &cur_snake_pos) {
                self.alive = false;
            }

            if crate::game_logic::Game::check_is_in_happy_collison(self, &cur_snake_pos) {
                self.main_actor.collected = self.main_actor.collected + 1;

                for i in 0..(self.main_actor.collected) as usize {
                    if self.sneak_peaces[i].index == -1_isize as usize {
                        self.sneak_peaces[i].index = 0;
                        continue;
                    }

                    self.sneak_peaces[i].index = self.sneak_peaces[i].index + 1;
                    
                    if self.sneak_peaces[i].index == (self.main_actor.collected - 1) as usize {
                        self.sneak_peaces[i].pixels.set_pos(last_pos);
                    }
                }
            }
            else if self.main_actor.collected == 1 {
                self.sneak_peaces[0].pixels.set_pos(last_pos);
            }
            else if self.main_actor.collected > 0 {
                for i in 0..(self.main_actor.collected) as usize {
                    if self.sneak_peaces[i].index == (self.main_actor.collected - 1) as usize {
                        self.sneak_peaces[i].index = 0;
                        continue;
                    }
                    if self.sneak_peaces[i].index == (self.main_actor.collected - 2) as usize {
                        self.sneak_peaces[i].pixels.set_pos(last_pos);
                    }

                    self.sneak_peaces[i].index = self.sneak_peaces[i].index + 1;
                }
            }

            self.tick = self.tick + 1;
            if self.tick == u64::max_value() {
                self.tick = 0;
            }
        }

        fn random_vec2i16() -> Vec2i16 {
            Vec2i16 { 
                x: (crate::game_logic::Game::random_number()),
                y: (crate::game_logic::Game::random_number()) }
        }

        fn random_number() -> i16 {
            use rand::Rng;
            rand::thread_rng().gen_range(0..100)
        }

        fn check_is_in_deadly_collison(&self, coord: &Vec2i16) -> bool {
            for i in self.world.walls.iter() {
                let wall_pos = i.get_pos();
                let wall_size = i.get_size();
                let x_diff = coord.x - wall_pos.x;
                let y_diff = coord.y - wall_pos.y;
    
                if x_diff < 0 || y_diff < 0 {
                    continue;
                }

                if x_diff < wall_size.x &&
                    y_diff < wall_size.y {
                    return true;
                }
            }
            for i in self.sneak_peaces.iter() {
                let wall_pos = i.pixels.get_pos();
                let wall_size = i.pixels.get_size();
                let x_diff = coord.x - wall_pos.x;
                let y_diff = coord.y - wall_pos.y;
    
                if x_diff < 0 || y_diff < 0 {
                    continue;
                }

                if x_diff < wall_size.x &&
                    y_diff < wall_size.y {
                    return true;
                }
            }
            false
        }

        fn check_is_in_happy_collison(&mut self, coord: &Vec2i16) -> bool {
            for i in self.apples.iter_mut() {
                let apple_pos = i.pixels.get_pos();
                let apple_size = i.pixels.get_size();
                let x_diff = coord.x - apple_pos.x;
                let y_diff = coord.y - apple_pos.y;
    
                if x_diff < 0 || y_diff < 0 {
                    continue;
                }

                if x_diff < apple_size.x &&
                    y_diff < apple_size.y {
                    i.alive = false;
                    i.pixels.set_pos(Vec2i16 { x: (-1), y: (-1) });
                    return true;
                }
            }
                
            false
        }
    }
}

mod term_input {
    use std::sync::atomic::Ordering;
    use std::thread::sleep;
    use std::time::Duration;

    use crate::{end_keyboard_hook_on_this_thread, set_up_keyboard_hook_on_this_thread};

    pub struct Input {
        handle: std::thread::JoinHandle<()>,
        pub last: std::sync::Arc<std::sync::atomic::AtomicU32>,
        loop_handle: std::sync::Arc<std::sync::atomic::AtomicBool>,
    }

    impl Input {
        pub fn initialize() -> Self {
            use crate::KEY;
            use std::thread;
            use std::sync::Arc;

            let mut r = Input {
                handle: thread::spawn(|| {}),
                last: Arc::new(('W' as u32).into()),
                loop_handle: Arc::new(true.into()),
            };
            let loop_handle_arc = r.loop_handle.clone();
            let last_key_arc = r.last.clone();

            let h = std::thread::spawn(move || {
                    use winapi::um::winuser::PeekMessageA;
                    use winapi::um::winuser::MSG;
                    use winapi::um::winuser::PM_REMOVE;
                    use winapi::shared::windef::HWND;
                    use winapi::shared::windef::POINT;
                    
                    let mut llkbd_hook_id = set_up_keyboard_hook_on_this_thread();
                    
                    let mut msg = MSG {
                        hwnd: 0 as HWND,
                        message: 0 as u32,
                        wParam: 0 as usize,
                        lParam: 0 as isize,
                        time: 0,
                        pt: POINT { x: 0, y: 0 },
                    };
                    loop {
                        unsafe {
                            PeekMessageA(&mut msg, 0 as HWND, 0, 0, PM_REMOVE); 
                        }

                        if !loop_handle_arc.load(Ordering::Relaxed) {
                            break;
                        }
                        
                        unsafe { last_key_arc.store(KEY, Ordering::Relaxed); }
                        sleep(Duration::from_millis(10));
                    }
                    
                    end_keyboard_hook_on_this_thread(&mut llkbd_hook_id);
                });
        
            r.handle = h;
            return r;
        }

        pub fn destroy(&mut self) {
            self.loop_handle.store(false, Ordering::Relaxed);
        }
    }
}

fn main() {
    use term_steady_out::Renderer;
    use term_steady_out::Render;
    use game_logic::Game;
    use term_input::Input;

    let mut x = Renderer::initialize();
    let mut i = Input::initialize();
    let mut g = Game::initialize(&mut x, &i);

    loop
    {
        sleep(Duration::from_millis(100));
        
        g.update();
        x.render();

        if !g.alive {
            break;
        }
    };

    i.destroy();

    println!("score {}", g.get_score());
}

