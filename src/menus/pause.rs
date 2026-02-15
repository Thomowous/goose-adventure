//! The pause menu.

use bevy::{input::common_conditions::input_just_pressed, prelude::*};

use crate::{menus::Menu, screens::Screen, theme::widget};

pub(super) fn plugin(app: &mut App) {

}

fn spawn_pause_menu(mut commands: Commands) {
    commands.spawn((
        widget::ui_root("Pause Menu"),
        GlobalZIndex(2),
        DespawnOnExit(Menu::Pause),
        children![
            widget::header("Game paused"),
            widget::button("Continue", close_menu),
            widget::button("Settings", open_settings_menu),
            widget::button("Quit to title", quit_to_title),
        ],
    ));
}

fn open_settings_menu(_: On<Pointer<Click>>, mut next_menu: If<ResMut<NextState<Menu>>>) {
    next_menu.set(Menu::Settings);
}

fn close_menu(_: On<Pointer<Click>>, mut next_menu: If<ResMut<NextState<Menu>>>) {
    next_menu.set(Menu::None);
}

fn quit_to_title(_: On<Pointer<Click>>, mut next_screen: If<ResMut<NextState<Screen>>>) {
    next_screen.set(Screen::Title);
}

fn go_back(mut next_menu: If<ResMut<NextState<Menu>>>) {
    next_menu.set(Menu::None);
}
