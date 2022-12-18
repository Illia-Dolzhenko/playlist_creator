use bmbf_utils::*;
use egui::Sense;

pub mod bmbf_utils;

struct App {
    custom_levels: Vec<CustomLevel>,
    available_levels: Vec<CustomLevel>,
    selected_level: Option<usize>,
    playlists: Vec<Playlist>,
    selected_playlist: Option<usize>,
    selected_song: Option<usize>,
}

#[derive(Debug, Clone)]
pub enum Message {
    Test(i32),
}

fn main() {
    let mut custom_levels = get_custom_levels();
    println!("CustomLevels size: {}", custom_levels.len());
    custom_levels.sort_by(|a, b| a.modified.cmp(&b.modified));
    for level in custom_levels.iter() {
        println!(
            "Song name: {}, modified: {}",
            level.song_name,
            level.modified.unwrap_or(0)
        );
    }
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(840., 480.)),
        ..Default::default()
    };

    let playlists = get_playlists();
    playlists.iter().for_each(|playlist| {
        println!("{}", playlist.title);
    });

    let p_songs: usize = playlists.iter().map(|playlist| playlist.songs.len()).sum();

    println!("Number of songs in all playlists: {}", p_songs);
    println!("Custom levels total: {}", custom_levels.len());

    let available_levels: Vec<CustomLevel> = custom_levels
        .iter()
        .cloned()
        .filter(|level| {
            !playlists.iter().any(|playlist| {
                playlist.songs.iter().any(|playlist_song| {
                    level.hash.is_some() && playlist_song.hash.eq(level.hash.as_ref().unwrap())
                })
            })
        })
        .collect();

    println!(
        "Custom levels that are not in any playlists: {}",
        available_levels.len()
    );

    eframe::run_native(
        "Playlist Creator",
        options,
        Box::new(|_cc| {
            Box::new(App {
                custom_levels,
                available_levels,
                playlists,
                selected_level: None,
                selected_playlist: None,
                selected_song: None,
            })
        }),
    );
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let text_style = egui::TextStyle::Body;
            let row_height = ui.text_style_height(&text_style);

            egui::SidePanel::left("left_panel")
                .resizable(true)
                .default_width(300.0)
                //.width_range(80.0..=440.0)
                .show_inside(ui, |ui| {
                    let current_level = match self.selected_level {
                        Some(level_index) => match self.available_levels.get(level_index) {
                            Some(level) => format!(
                                "{} by {}",
                                level.song_name.to_owned(),
                                level.song_author.to_owned()
                            ),
                            None => "Unknown".to_string(),
                        },
                        None => "Select level".to_string(),
                    };
                    ui.vertical_centered(|ui| {
                        ui.button("Force reload");
                        ui.heading(current_level);
                    });
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        egui::ScrollArea::vertical().show_rows(
                            ui,
                            row_height,
                            self.available_levels.len(),
                            |ui, range| {
                                for row in range {
                                    let level = self.available_levels.get(row);
                                    let (song_name, author) = match level {
                                        Some(level) => {
                                            (level.song_name.as_str(), level.song_author.as_str())
                                        }
                                        None => ("Unknown", "Unknown"),
                                    };
                                    let text = format!("{} by {}", song_name, author);
                                    if ui
                                        .add(egui::Label::new(&text).sense(Sense::click()))
                                        .clicked()
                                    {
                                        self.selected_level = Some(row);
                                    }
                                }
                            },
                        );
                    });
                });

            egui::SidePanel::right("right_panel")
                .resizable(true)
                .default_width(300.0)
                //.width_range(80.0..=440.0)
                .show_inside(ui, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.horizontal(|ui| {
                            ui.button("+");
                            ui.button("Save to device");
                        });

                        ui.heading("Playlists:");
                    });

                    egui::ScrollArea::vertical().show_rows(
                        ui,
                        row_height,
                        self.playlists.len(),
                        |ui, range| {
                            for row in range {
                                if let Some(playlist) = self.playlists.get(row) {
                                    //ui.heading(playlist.title.to_string());
                                    if ui
                                        .add(
                                            egui::Label::new(playlist.title.to_string())
                                                .sense(Sense::click()),
                                        )
                                        .clicked()
                                    {
                                        self.selected_playlist = Some(row);
                                        self.selected_song = None;
                                    }
                                }
                                //ui.heading("some playlist");
                            }
                        },
                    );
                });

            egui::TopBottomPanel::bottom("bottom_panel")
                .resizable(false)
                .min_height(0.0)
                .show_inside(ui, |ui| {
                    ui.horizontal_centered(|ui| {
                        //ui.heading("Bottom Panel");
                        if ui.button(">>").clicked() {
                            self.add_selected_song_to_selected_playlist();
                            self.selected_level = None;
                        }
                        if ui.button("X").clicked() {
                            self.remove_selected_song_from_selected_playlist();
                        }
                    });
                });

            egui::CentralPanel::default().show_inside(ui, |ui| {
                // let selected_playlist = match self.selected_playlist {
                //     Some(index) => match self.playlists.get(index) {
                //         Some(playlist) => playlist.title.to_string(),
                //         None => "Unknown".to_string(),
                //     },
                //     None => "Select playlist".to_string(),
                // };
                if let Some(selected_playlist_index) = self.selected_playlist {
                    if let Some(playlist) = self.playlists.get(selected_playlist_index) {
                        ui.vertical_centered(|ui| {
                            ui.heading(playlist.title.to_string());
                            if let Some(song_index) = self.selected_song {
                                if let Some(song) = playlist.songs.get(song_index) {
                                    ui.heading(song.name.to_string());
                                }
                            }
                        });
                        egui::ScrollArea::vertical().show_rows(
                            ui,
                            row_height,
                            playlist.songs.len(),
                            |ui, range| {
                                for row in range {
                                    if let Some(song) = playlist.songs.get(row) {
                                        if ui
                                            .add(
                                                egui::Label::new(song.name.to_string())
                                                    .sense(Sense::click()),
                                            )
                                            .clicked()
                                        {
                                            self.selected_song = Some(row)
                                        }
                                        //ui.label(song.name.to_string());
                                    }
                                    //ui.heading("some song");
                                }
                            },
                        );
                    }
                }
            });
        });
    }
}

impl App {
    fn add_selected_song_to_selected_playlist(&mut self) {
        if let (Some(playlist_index), Some(level_index)) =
            (self.selected_playlist, self.selected_level)
        {
            if let (Some(playlist), Some(level)) = (
                self.playlists.get_mut(playlist_index),
                self.available_levels.get(level_index),
            ) {
                playlist.songs.push(Song {
                    name: level.song_name.to_string(),
                    hash: level
                        .hash
                        .as_ref()
                        .unwrap_or(&"Unknown".to_string())
                        .to_string(),
                });

                self.available_levels.remove(level_index);
            }
        }
    }

    fn remove_selected_song_from_selected_playlist(&mut self) {
        if let (Some(playlist_index), Some(song_index)) =
            (self.selected_playlist, self.selected_song)
        {
            if let Some(playlist) = self.playlists.get_mut(playlist_index) {
                if let Some(_song) = playlist.songs.get(song_index) {
                    let song = playlist.songs.remove(song_index);
                    if let Some(custom_level) = self.custom_levels.iter().find(|level| {
                        level
                            .hash
                            .as_ref()
                            .unwrap_or(&"unknown".to_string())
                            .eq(&song.hash)
                    }) {
                        self.available_levels.push(custom_level.clone());
                    }
                }
            }
        }
    }
}
