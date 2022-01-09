// debug related controls
use bevy::{
    diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
    prelude::*,
};

const FONT_SIZE: f32 = 40.;
const FONT_BOLD: &str = "fonts/FiraSans-Bold.ttf";
const FONT_MEDIUM: &str = "fonts/FiraMono-Medium.ttf";

#[derive(Component)]
struct FPSLabel;

#[derive(Component)]
struct Console;

#[derive(Component)]
struct ConsoleWindow;

#[derive(Component)]
struct ConsoleLabel;

#[derive(Component)]
struct ConsoleCommand;

struct DebugParams {
    console_input: String,
    command_queue: Vec<String>,
}

impl Default for DebugParams {
    fn default() -> Self {
        Self {
            console_input: String::new(),
            command_queue: vec![],
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
enum FPSState {
    On,
    Off,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
enum ConsoleState {
    On,
    Off,
}

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(FrameTimeDiagnosticsPlugin::default())
            .insert_resource(DebugParams::default())
            .add_startup_system(setup_debug_ui)
            .add_startup_system(setup_fps)
            .add_startup_system(setup_console_window)
            .add_state(ConsoleState::Off)
            .add_state(FPSState::Off)
            .add_system_set(SystemSet::on_enter(FPSState::On).with_system(update_fps_visibility))
            .add_system_set(SystemSet::on_update(FPSState::On).with_system(update_fps))
            .add_system_set(SystemSet::on_enter(FPSState::Off).with_system(update_fps_visibility))
            .add_system_set(
                SystemSet::on_enter(ConsoleState::On).with_system(update_console_visibility),
            )
            .add_system_set(
                SystemSet::on_update(ConsoleState::On)
                    .with_system(update_console_input)
                    .with_system(commands_processor),
            )
            .add_system_set(
                SystemSet::on_enter(ConsoleState::Off).with_system(update_console_visibility),
            )
            .add_system(exit_control)
            .add_system(toggle_console);
    }
}

fn setup_debug_ui(mut commands: Commands) {
    commands.spawn_bundle(UiCameraBundle::default());
}

fn exit_control(input: Res<Input<KeyCode>>) {
    if input.just_pressed(KeyCode::Escape) {
        std::process::exit(0);
    }
}

fn toggle_console(input: Res<Input<KeyCode>>, mut console_state: ResMut<State<ConsoleState>>) {
    if input.just_pressed(KeyCode::F1) {
        match console_state.current() {
            ConsoleState::On => {
                console_state.set(ConsoleState::Off).unwrap();
            }
            ConsoleState::Off => {
                console_state.set(ConsoleState::On).unwrap();
            }
        }
    }
}

fn setup_fps(mut commands: Commands, asset_server: Res<AssetServer>) {
    let style = Style {
        align_self: AlignSelf::FlexEnd,
        margin: Rect {
            left: Val::Px(10.),
            ..Default::default()
        },
        ..Default::default()
    };

    let fps_field = TextSection {
        value: "FPS: ".to_string(),
        style: TextStyle {
            font_size: FONT_SIZE,
            color: Color::YELLOW,
            font: asset_server.load(FONT_BOLD),
        },
    };

    let fps_text = TextSection {
        value: "".to_string(),
        style: TextStyle {
            font_size: FONT_SIZE,
            color: Color::GOLD,
            font: asset_server.load(FONT_MEDIUM),
        },
    };

    commands
        .spawn_bundle(TextBundle {
            style,
            text: Text {
                sections: vec![fps_field, fps_text],
                ..Default::default()
            },
            visibility: Visibility { is_visible: false },
            ..Default::default()
        })
        .insert(FPSLabel);
}

fn update_fps(diagnostics: Res<Diagnostics>, mut query: Query<&mut Text, With<FPSLabel>>) {
    for mut text in query.iter_mut() {
        if let Some(fps) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS) {
            if let Some(average) = fps.average() {
                // Update the value of the second section
                text.sections[1].value = format!("{:.2}", average);
                println!("FPS: {:.2}", average);
            }
        }
    }
}

fn update_fps_visibility(
    fps_state: Res<State<FPSState>>,
    mut query: Query<&mut Visibility, With<FPSLabel>>,
) {
    let mut v = query.single_mut();
    match fps_state.current() {
        FPSState::On => {
            v.is_visible = true;
        }
        FPSState::Off => {
            v.is_visible = false;
        }
    }
}

fn setup_console_window(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    debug_params: ResMut<DebugParams>,
) {
    // Draw a console window at bottom of screen like UE4
    // with a prefix '>' on the left
    let window_style = Style {
        position_type: PositionType::Absolute,
        align_self: AlignSelf::FlexStart,
        size: Size::new(Val::Percent(100.0), Val::Px(30.0)),
        ..Default::default()
    };
    let label_style = Style {
        margin: Rect {
            left: Val::Px(5.),
            right: Val::Px(10.),
            top: Val::Px(5.),
            bottom: Val::Px(5.),
        },
        ..Default::default()
    };
    let command_style = Style {
        margin: Rect {
            left: Val::Px(20.),
            right: Val::Px(10.),
            ..Default::default()
        },
        ..Default::default()
    };
    commands
        .spawn_bundle(NodeBundle {
            style: window_style,
            color: Color::GRAY.into(),
            visibility: Visibility { is_visible: false },
            ..Default::default()
        })
        .with_children(|parent| {
            parent
                .spawn_bundle(TextBundle {
                    style: label_style,
                    visibility: Visibility { is_visible: false },
                    text: Text::with_section(
                        ">",
                        TextStyle {
                            font_size: 20.,
                            font: asset_server.load(FONT_BOLD),
                            color: Color::YELLOW,
                            ..Default::default()
                        },
                        Default::default(),
                    ),
                    ..Default::default()
                })
                .insert(Console)
                .insert(ConsoleLabel)
                .with_children(|parent| {
                    parent
                        .spawn_bundle(TextBundle {
                            style: command_style.clone(),
                            visibility: Visibility { is_visible: false },
                            text: Text::with_section(
                                debug_params.console_input.clone(),
                                TextStyle {
                                    font_size: 20.,
                                    color: Color::WHITE,
                                    font: asset_server.load(FONT_BOLD),
                                    ..Default::default()
                                },
                                Default::default(),
                            ),
                            ..Default::default()
                        })
                        .insert(Console)
                        .insert(ConsoleCommand);
                });
        })
        .insert(Console)
        .insert(ConsoleWindow);
}

fn update_console_visibility(
    console_state: Res<State<ConsoleState>>,
    mut query: Query<&mut Visibility, With<Console>>,
) {
    let visibilities = query.iter_mut();

    for mut v in visibilities {
        match console_state.current() {
            ConsoleState::On => {
                v.is_visible = true;
            }
            ConsoleState::Off => {
                v.is_visible = false;
            }
        }
    }
}

fn update_console_input(
    mut debug_params: ResMut<DebugParams>,
    mut input: EventReader<ReceivedCharacter>,
    mut query: Query<&mut Text, With<ConsoleCommand>>,
) {
    let mut text = query.single_mut();
    for e in input.iter() {
        if e.char == '\u{0a}' || e.char == '\u{0d}' {
            // push new command to queue
            let command = debug_params.console_input.clone();
            debug_params.command_queue.push(command);
            debug_params.console_input.clear();
        } else if e.char == '\u{8}' {
            let mut modified = debug_params.console_input.chars();
            modified.next_back();
            debug_params.console_input = modified.as_str().to_string();
        } else {
            debug_params.console_input += &e.char.to_string();
        }

        text.sections[0].value = debug_params.console_input.clone();
    }
}

fn commands_processor(
    mut debug_params: ResMut<DebugParams>,
    mut fps_state: ResMut<State<FPSState>>,
) {
    if debug_params.command_queue.len() > 0 {
        let command = debug_params.command_queue.first().unwrap().clone();
        // remove first command in the queue
        debug_params.command_queue = debug_params.command_queue.drain(1..).collect();
        // process the command
        let cmd = command.to_uppercase();
        match cmd.as_str() {
            "FPS" => match fps_state.current() {
                FPSState::Off => {
                    fps_state.set(FPSState::On).unwrap();
                }
                FPSState::On => {
                    fps_state.set(FPSState::Off).unwrap();
                }
            },
            _ => {}
        }
    }
}
