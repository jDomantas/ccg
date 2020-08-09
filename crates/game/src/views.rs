pub mod main;
pub mod settings;
pub mod game;
pub mod card_list;
pub mod card_select;

use engine::{ggez, Ctx, FrameRenderer, Result};
use engine::ggez::graphics::{DrawParam, Scale, Text, TextFragment};
use crate::GameData;

pub use self::main::MainMenu;
pub use self::settings::Settings;
pub use self::game::GameState;
pub use self::card_list::CardList;
pub use self::card_select::CardSelect;

#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub enum DrawKind {
    Opaque,
    OnTop,
}

pub enum ViewChange {
    None,
    Push(Box<dyn View>),
    Pop,
    Replace(Box<dyn View>),
}

pub trait View {
    fn draw_kind(&self) -> DrawKind;
    fn update(&mut self, data: &GameData, ctx: &mut Ctx<'_>, dt: f32) -> Result<ViewChange>;
    fn draw(&mut self, renderer: &mut FrameRenderer<'_>) -> Result;
}

pub struct ViewStack {
    views: Vec<Box<dyn View>>,
}

impl ViewStack {
    pub fn new(view: impl View + 'static) -> Self {
        ViewStack {
            views: vec![
                Box::new(view),
            ],
        }
    }

    pub fn update(&mut self, data: &GameData, ctx: &mut Ctx<'_>, dt: f32) -> Result {
        let view = self.views.last_mut().expect("view stack empty");
        match view.update(data, ctx, dt)? {
            ViewChange::None => {}
            ViewChange::Push(view) => self.views.push(view),
            ViewChange::Pop => {
                assert!(self.views.len() >= 2, "can't pop last view");
                self.views.pop();
            }
            ViewChange::Replace(new) => *view = new,
        }
        Ok(())
    }

    pub fn draw(&mut self, renderer: &mut FrameRenderer<'_>) -> Result {
        self.draw_at(renderer, self.views.len() - 1)
    }

    fn draw_at(&mut self, renderer: &mut FrameRenderer<'_>, idx: usize) -> Result {
        match self.views[idx].draw_kind() {
            DrawKind::Opaque => {}
            DrawKind::OnTop if idx == 0 => {
                panic!("first view wants to be drawn on top of nothing");
            }
            DrawKind::OnTop => {
                self.draw_at(renderer, idx - 1)?;
            }
        }
        self.views[idx].draw(renderer)
    }
}

const BUTTON_WIDTH: f32 = 1000.0;
const BUTTON_HEIGHT: f32 = 80.0;
const SLIDE_LENGTH: f32 = 30.0;
const INDENT_LENGTH: f32 = 100.0;

pub enum ButtonState {
    Normal,
    Selected,
}

struct Button<I> {
    text: Text,
    spec: ButtonSpec<I>,
    slide: f32,
    hover: bool,
}

impl<I> Button<I> {
    fn from_spec(spec: ButtonSpec<I>) -> Self {
        let fragment = TextFragment::new(spec.text).scale(Scale::uniform(70.0));
        let text = Text::new(fragment);
        Button {
            text,
            spec,
            slide: 0.0,
            hover: false,
        }
    }

    fn draw(&self, renderer: &mut engine::FrameRenderer<'_>, x: f32, y: f32) -> Result {
        let x = x + self.slide * SLIDE_LENGTH;
        let (x, y) = renderer.ctx().round_to_screen((x, y));
        let textures = renderer.textures();
        let texture = match self.spec.state {
            ButtonState::Normal if self.hover => textures.button_hover,
            ButtonState::Normal => textures.button,
            ButtonState::Selected => textures.button_selected,
        };
        renderer.draw(texture, x, y, BUTTON_WIDTH, BUTTON_HEIGHT)?;
        // ggez::graphics::draw(renderer.ggez(), &self.text, DrawParam::new()
        //     .dest([x + 10.0, y + 5.0])
        //     .color(ggez::graphics::BLACK))
        ggez::graphics::queue_text(renderer.ggez(), &self.text, [x + 10.0, y + 5.0], Some(ggez::graphics::BLACK));
        Ok(())
    }
}

pub struct ButtonSpec<I> {
    pub text: &'static str,
    pub state: ButtonState,
    pub on_click: I,
    pub indent_level: u32,
}

impl<I> ButtonSpec<I> {
    fn matches(&self, other: &Self) -> bool {
        (self.text, self.indent_level) == (other.text, other.indent_level)
    }
}

pub trait MenuSpec: Sized {
    type Input: Clone;

    fn top_padding() -> f32 { 0.0 }
    fn update(&mut self, data: &GameData, ctx: &mut Ctx<'_>, action: Self::Input) -> Result<ViewChange>;
    fn create_elements(&self) -> Vec<ButtonSpec<Self::Input>>;
}

fn make_buttons<I>(specs: Vec<ButtonSpec<I>>) -> Vec<Button<I>> {
    specs
        .into_iter()
        .map(Button::from_spec)
        .collect()
}

fn merge_buttons<I>(mut old: Vec<Button<I>>, mut new: Vec<ButtonSpec<I>>) -> Vec<Button<I>> {
    let mut new_buttons = Vec::new();
    while let Some(spec) = new.pop() {
        if old.iter().any(|b| b.spec.matches(&spec)) {
            while let Some(mut btn) = old.pop() {
                if btn.spec.matches(&spec) {
                    btn.spec = spec;
                    new_buttons.push(btn);
                    break;
                }
            }
        } else {
            new_buttons.push(Button::from_spec(spec));
        }
    }
    new_buttons.reverse();
    new_buttons
}

pub struct MenuView<S: MenuSpec> {
    spec: S,
    current_buttons: Vec<Button<S::Input>>,
}

impl<S: MenuSpec> MenuView<S> {
    pub fn new(spec: S) -> Self {
        let current_buttons = make_buttons(spec.create_elements());
        MenuView {
            spec,
            current_buttons,
        }
    }
    
    fn update_spec(&mut self, data: &GameData, ctx: &mut Ctx<'_>, input: S::Input) -> Result<ViewChange> {
        let change = self.spec.update(data, ctx, input)?;
        let new_buttons = self.spec.create_elements();
        self.current_buttons = merge_buttons(
            std::mem::take(&mut self.current_buttons),
            new_buttons,
        );
        Ok(change)
    }

    fn buttons_with_positions(&mut self) -> impl Iterator<Item = (&mut Button<S::Input>, f32, f32)> + '_ {
        let x = 80.0;
        let mut y = 80.0 + S::top_padding();
        self.current_buttons.iter_mut()
            .map(move |b| {
                let x = x + INDENT_LENGTH * b.spec.indent_level as f32;
                let item = (b, x, y);
                y += BUTTON_HEIGHT * 1.4;
                item
            })
    }
}

impl<S: MenuSpec> View for MenuView<S> {
    fn draw_kind(&self) -> DrawKind {
        DrawKind::Opaque
    }

    fn update(&mut self, data: &GameData, ctx: &mut Ctx<'_>, dt: f32) -> Result<ViewChange> {
        for button in &mut self.current_buttons {
            let delta_slide = if button.hover { 1.0 } else { -1.0 };
            let slide = (button.slide + delta_slide * dt * 10.0);
            let slide = if slide < 0.0 { 0.0 } else if slide > 1.0 { 1.0 } else { slide };
            button.slide = slide;
        }
        let (mouse_x, mouse_y) = ctx.mouse_position();
        let mut input = None;
        for (button, x, y) in self.buttons_with_positions() {
            let inside = mouse_x >= x &&
                mouse_y >= y &&
                mouse_x < x + BUTTON_WIDTH &&
                mouse_y < y + BUTTON_HEIGHT;
            button.hover = inside;
            if inside && ctx.is_mouse_click() {
                input = Some(button.spec.on_click.clone());
                break;
            }
        }
        if let Some(input) = input {
            self.update_spec(data, ctx, input)
        } else {
            Ok(ViewChange::None)
        }
    }

    fn draw(&mut self, renderer: &mut FrameRenderer<'_>) -> Result {
        for (button, x, y) in self.buttons_with_positions() {
            button.draw(renderer, x, y)?;
        }
        ggez::graphics::draw_queued_text(renderer.ggez(), DrawParam::default(), None, ggez::graphics::FilterMode::Linear)?;
        Ok(())
    }    
}
