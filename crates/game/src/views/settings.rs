use engine::{Ctx, Result, WindowMode};
use crate::GameData;
use super::{ButtonSpec, ButtonState, MenuSpec, MenuView, ViewChange};

pub struct Settings {
    resolutions: Vec<(f32, f32, &'static str, bool)>,
    modes: Vec<(WindowMode, &'static str, bool)>,
}

impl Settings {
    pub fn new() -> Settings {
        Settings {
            resolutions: Vec::new(),
            modes: Vec::new(),
        }
    }
}

#[derive(Copy, Clone)]
pub enum Input {
    SetResolution(f32, f32),
    PickResolution,
    SetWindowMode(WindowMode),
    PickWindowMode,
    Back,
}

impl Settings {
    fn recalculate_resolutions(&mut self, cw: f32, ch: f32) {
        self.resolutions.clear();
        self.modes.clear();
        for &(w, h, text) in ALLOWED_RESOLUTIONS {
            let selected = (cw - w).abs() <= 1.0 && (ch - h).abs() <= 1.0;
            self.resolutions.push((w, h, text, selected));
        }
    }

    fn recalculate_modes(&mut self, mode: WindowMode) {
        self.resolutions.clear();
        self.modes.clear();
        for &(m, text) in ALLOWED_MODES {
            let selected = m == mode;
            self.modes.push((m, text, selected));
        }
    }
}

impl MenuSpec for Settings {
    type Input = Input;

    fn update(&mut self, data: &GameData, ctx: &mut Ctx<'_>, input: Self::Input) -> Result<ViewChange> {
        match input {
            Input::SetResolution(w, h) => {
                ctx.set_window_size(w, h)?;
                self.recalculate_resolutions(w, h);
            }
            Input::PickResolution => {
                let (w, h) = ctx.window_size();
                self.recalculate_resolutions(w, h);
            }
            Input::SetWindowMode(mode) => {
                ctx.set_window_mode(mode)?;
                self.recalculate_modes(mode);
            }
            Input::PickWindowMode => {
                let mode = ctx.window_mode();
                self.recalculate_modes(mode);
            }
            Input::Back => {
                return Ok(ViewChange::Pop);
            }
        }
        Ok(ViewChange::None)
    }

    fn create_elements(&self) -> Vec<ButtonSpec<Self::Input>> {
        let mut buttons = Vec::new();

        buttons.push(ButtonSpec {
            text: "Screen resolution",
            state: if self.resolutions.len() > 0 { ButtonState::Selected } else { ButtonState::Normal },
            on_click: Input::PickResolution,
            indent_level: 0,
        });
        for &(w, h, text, selected) in &self.resolutions {
            buttons.push(ButtonSpec {
                text,
                state: if selected { ButtonState::Selected } else { ButtonState::Normal },
                on_click: Input::SetResolution(w, h),
                indent_level: 1,
            });
        }

        buttons.push(ButtonSpec {
            text: "Window mode",
            state: if self.modes.len() > 0 { ButtonState::Selected } else { ButtonState::Normal },
            on_click: Input::PickWindowMode,
            indent_level: 0,
        });
        for &(mode, text, selected) in &self.modes {
            buttons.push(ButtonSpec {
                text,
                state: if selected { ButtonState::Selected } else { ButtonState::Normal },
                on_click: Input::SetWindowMode(mode),
                indent_level: 1,
            });
        }

        buttons.push(ButtonSpec {
            text: "Back",
            state: ButtonState::Normal,
            on_click: Input::Back,
            indent_level: 0,
        });
        buttons
    }
}

const ALLOWED_RESOLUTIONS: &[(f32, f32, &str)] = &[
    (1280.0, 720.0, "1280 x 720"),
    (1600.0, 900.0, "1600 x 900"),
    (1920.0, 1080.0, "1920 x 1080"),
    (3840.0, 2160.0, "3840 x 2160"),
];

const ALLOWED_MODES: &[(WindowMode, &str)] = &[
    (WindowMode::Windowed, "Windowed"),
    (WindowMode::Borderless, "Borderless fullscreen"),
];
