use engine::{Ctx, Result};
use crate::GameData;
use super::{ButtonSpec, ButtonState, MenuSpec, MenuView, ViewChange};
use super::settings::Settings;

#[derive(Clone)]
pub enum Input {
    Play,
    Settings,
}

pub struct MainMenu;

impl MenuSpec for MainMenu {
    type Input = Input;

    fn top_padding() -> f32 {
        360.0
    }

    fn update(&mut self, data: &GameData, ctx: &mut Ctx<'_>, input: Self::Input) -> Result<ViewChange> {
        Ok(match input {
            Input::Play => ViewChange::Replace(Box::new(super::game::GameState::new(&data.decks))),
            Input::Settings => ViewChange::Push(Box::new(MenuView::new(Settings::new()))),
        })
    }

    fn create_elements(&self) -> Vec<ButtonSpec<Self::Input>> {
        vec![
            ButtonSpec {
                text: "Play",
                state: ButtonState::Normal,
                on_click: Input::Play,
                indent_level: 0,
            },
            ButtonSpec {
                text: "Settings",
                state: ButtonState::Normal,
                on_click: Input::Settings,
                indent_level: 0,
            },
            // ButtonSpec {
            //     text: "Quit",
            //     state: ButtonState::Normal,
            //     on_click: Action::ChangeView(|| ViewChange::None),
            //     indent_level: 0,
            // },
        ]
    }
}
