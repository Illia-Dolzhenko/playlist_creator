use levenshtein::levenshtein;
use std::fmt;

use bmbf_utils::*;

pub mod bmbf_utils;

#[derive(PartialEq)]
enum Sorting {
    BPMDsc,
    BPMAsc,
    NameDsc,
    NameAsc,
    ModifiedDsc,
    ModifiedAsc,
}

struct App {
    custom_levels: Vec<CustomLevel>,
    available_levels: Vec<CustomLevel>,
    selected_level: Option<usize>,
    playlists: Vec<Playlist>,
    selected_playlist: Option<usize>,
    selected_song: Option<usize>,
    text_input: String,
    level_search: String,
    create_new_playlist: bool,
    sort: Sorting,
}

fn main() {
    let mut custom_levels = get_custom_levels();
    println!("CustomLevels size: {}", custom_levels.len());
    //custom_levels.sort_by(|a, b| a.modified.cmp(&b.modified));
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

    let mut available_levels: Vec<CustomLevel> = custom_levels
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

    available_levels.sort_by(|level_1, level_2| level_1.modified.cmp(&level_2.modified));
    available_levels.reverse();

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
                text_input: "name".to_owned(),
                level_search: "".to_owned(),
                create_new_playlist: false,
                sort: Sorting::ModifiedDsc,
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
                .show_inside(ui, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.horizontal(|ui| {
                            egui::ComboBox::from_id_source("sorting_combo_box")
                                .selected_text(self.sort.to_string())
                                .show_ui(ui, |ui| {
                                    let responses = [
                                        ui.selectable_value(
                                            &mut self.sort,
                                            Sorting::BPMAsc,
                                            "BPM Ascending",
                                        ),
                                        ui.selectable_value(
                                            &mut self.sort,
                                            Sorting::BPMDsc,
                                            "BPM Descending",
                                        ),
                                        ui.selectable_value(
                                            &mut self.sort,
                                            Sorting::NameAsc,
                                            "Name Ascending",
                                        ),
                                        ui.selectable_value(
                                            &mut self.sort,
                                            Sorting::NameDsc,
                                            "Name Descending",
                                        ),
                                        ui.selectable_value(
                                            &mut self.sort,
                                            Sorting::ModifiedAsc,
                                            "Created Ascending",
                                        ),
                                        ui.selectable_value(
                                            &mut self.sort,
                                            Sorting::ModifiedDsc,
                                            "Created Descending",
                                        ),
                                    ];
                                    if responses.iter().any(|response| response.clicked()) {
                                        self.sort();
                                    }
                                });

                            ui.button("Force reload");
                        });
                        ui.horizontal(|ui| {
                            let search_response =
                                ui.add(egui::TextEdit::singleline(&mut self.level_search));

                            if !self.level_search.is_empty() && search_response.changed() {
                                self.levenshtein_sort();
                            }
                        });
                    });
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        egui::ScrollArea::vertical().show_rows(
                            ui,
                            row_height,
                            self.available_levels.len(),
                            |ui, range| {
                                for row in range {
                                    let text = self
                                        .available_levels
                                        .get(row)
                                        .map(|level| {
                                            format!(
                                                "{}, {} by: {}",
                                                level.beats_per_minute as i32,
                                                level.song_name.to_owned(),
                                                level.song_author.to_owned()
                                            )
                                        })
                                        .unwrap_or_else(|| "Unknown".to_owned());

                                    let is_selected = self
                                        .selected_level
                                        .map(|index| index == row)
                                        .unwrap_or_else(|| false);

                                    if ui
                                        .add(
                                            egui::SelectableLabel::new(is_selected, &text), //.sense(Sense::click()),
                                        )
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
                .show_inside(ui, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.horizontal(|ui| {
                            if ui.button("+").clicked() {
                                self.create_new_playlist = true;
                            }
                            if let Some(selected_playlist) = self.get_selected_playlist() {
                                if selected_playlist.just_created && ui.button("-").clicked() {
                                    self.remove_selected_playlist();
                                }
                            }

                            if ui.button("Save to device").clicked() {
                                save_modified_playlists(&self.playlists);
                            }
                        });
                        if self.create_new_playlist {
                            ui.add(egui::TextEdit::singleline(&mut self.text_input));
                            ui.horizontal(|ui| {
                                if ui.button("Cancel").clicked() {
                                    self.create_new_playlist = false;
                                }

                                if ui.button("Add").clicked() {
                                    self.create_new_playlist();
                                }
                            });
                        }
                        ui.heading("Playlists:");
                    });

                    egui::ScrollArea::vertical().show_rows(
                        ui,
                        row_height,
                        self.playlists.len(),
                        |ui, range| {
                            for row in range {
                                if let Some(playlist) = self.playlists.get(row) {
                                    let is_selected = self
                                        .selected_playlist
                                        .map(|index| index == row)
                                        .unwrap_or_else(|| false);

                                    if ui
                                        .add(egui::SelectableLabel::new(
                                            is_selected,
                                            playlist.title.to_string(),
                                        ))
                                        .clicked()
                                    {
                                        self.selected_playlist = Some(row);
                                        self.selected_song = None;
                                    }
                                }
                            }
                        },
                    );
                });

            egui::TopBottomPanel::bottom("bottom_panel")
                .resizable(false)
                .min_height(0.0)
                .show_inside(ui, |ui| {
                    ui.horizontal_centered(|ui| {
                        if ui.button(">>").clicked() {
                            self.add_selected_song_to_selected_playlist();
                            self.selected_level = None;
                        }
                        if ui.button("X").clicked() {
                            self.remove_selected_song_from_selected_playlist();
                            self.selected_song = None;
                        }
                    });
                });

            egui::CentralPanel::default().show_inside(ui, |ui| {
                if let Some(playlist) = self
                    .selected_playlist
                    .and_then(|index| self.playlists.get(index))
                {
                    ui.vertical_centered(|ui| {
                        ui.heading(playlist.title.to_string());
                    });
                    egui::ScrollArea::vertical().show_rows(
                        ui,
                        row_height,
                        playlist.songs.len(),
                        |ui, range| {
                            for row in range {
                                if let Some(song) = playlist.songs.get(row) {
                                    let is_selected = self
                                        .selected_song
                                        .map(|index| index == row)
                                        .unwrap_or_else(|| false);

                                    if ui
                                        .add(egui::SelectableLabel::new(
                                            is_selected,
                                            song.name.to_string(),
                                        ))
                                        .clicked()
                                    {
                                        self.selected_song = Some(row)
                                    }
                                }
                            }
                        },
                    );
                }
            });
        });
    }
}

impl App {
    fn levenshtein_sort(&mut self) {
        self.available_levels.sort_by(|level_1, level_2| {
            levenshtein(&level_1.song_name, &self.level_search)
                .cmp(&levenshtein(&level_2.song_name, &self.level_search))
        })
    }

    fn sort(&mut self) {
        match self.sort {
            Sorting::BPMDsc => {
                self.sort_bpm();
                self.available_levels.reverse();
            }
            Sorting::BPMAsc => self.sort_bpm(),
            Sorting::NameDsc => {
                self.sort_name();
                self.available_levels.reverse();
            }
            Sorting::NameAsc => self.sort_name(),
            Sorting::ModifiedDsc => {
                self.sort_modified();
                self.available_levels.reverse();
            }
            Sorting::ModifiedAsc => self.sort_modified(),
        }
    }

    fn sort_bpm(&mut self) {
        self.available_levels.sort_by(|level_1, level_2| {
            level_1
                .beats_per_minute
                .total_cmp(&level_2.beats_per_minute)
        });
    }

    fn sort_name(&mut self) {
        self.available_levels
            .sort_by(|level_1, level_2| level_1.song_name.cmp(&level_2.song_name));
    }

    fn sort_modified(&mut self) {
        self.available_levels
            .sort_by(|level_1, level_2| level_1.modified.cmp(&level_2.modified))
    }

    fn create_new_playlist(&mut self) {
        let title = self.text_input.to_string();

        if !self
            .playlists
            .iter()
            .any(|playlist| playlist.title.eq(&title))
        {
            let new_playlist = Playlist {
                file_name: format!("{}.json", title),
                changed: true,
                just_created: true,
                songs: Vec::new(),
                title,
                ..Default::default()
            };
            self.playlists.push(new_playlist);
            self.create_new_playlist = false;
        } else {
            println!("Playlist with the same title already exists!");
        }
    }

    fn get_selected_playlist(&self) -> Option<&Playlist> {
        self.selected_playlist
            .and_then(|index| self.playlists.get(index))
    }

    fn remove_selected_playlist(&mut self) {
        if let Some(playlist) = self.get_selected_playlist() {
            let mut levels: Vec<CustomLevel> = playlist
                .songs
                .iter()
                .flat_map(|song| self.find_level_by_hash(&song.hash))
                .collect();
            self.available_levels.append(&mut levels);
            self.sort();
        }

        if let Some(index) = self.selected_playlist {
            self.playlists.remove(index);
            self.selected_playlist = None;
        }
    }

    fn find_level_by_hash(&self, hash: &str) -> Option<CustomLevel> {
        self.custom_levels
            .iter()
            .find(|level| {
                if let Some(level_hash) = &level.hash {
                    level_hash.eq(hash)
                } else {
                    false
                }
            })
            .cloned()
    }

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

                playlist.changed = true;
                self.available_levels.remove(level_index);
            }
        }
    }

    fn remove_selected_song_from_selected_playlist(&mut self) {
        if let (Some(playlist_index), Some(song_index)) =
            (self.selected_playlist, self.selected_song)
        {
            // if let Some(playlist) = self.playlists.get_mut(playlist_index) {
            //     if let Some(_song) = playlist.songs.get(song_index) {
            //         let song = playlist.songs.remove(song_index);
            //         if let Some(custom_level) = self.custom_levels.iter().find(|level| {
            //             level
            //                 .hash
            //                 .as_ref()
            //                 .unwrap_or(&"unknown".to_string())
            //                 .eq(&song.hash)
            //         }) {
            //             self.available_levels.push(custom_level.clone());
            //         }
            //     }
            // }
            // self.selected_playlist.zip(self.selected_song).iter().for_each(|(playlist_index, song_index)|{

            // });

            self.playlists
                .get_mut(playlist_index)
                .map(|playlist| {
                    playlist.changed = true;
                    playlist.songs.remove(song_index)
                })
                .and_then(|song| {
                    self.custom_levels.iter().find(|level| {
                        level
                            .hash
                            .as_ref()
                            .unwrap_or(&"unknown".to_string())
                            .eq(&song.hash)
                    })
                })
                .into_iter()
                .for_each(|level| {
                    self.available_levels.push(level.clone());
                })
        }
    }
}

impl fmt::Display for Sorting {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Sorting::BPMDsc => write!(f, "BPM \\/"),
            Sorting::BPMAsc => write!(f, "BPM /\\"),
            Sorting::NameDsc => write!(f, "Name \\/"),
            Sorting::NameAsc => write!(f, "Name /\\"),
            Sorting::ModifiedDsc => write!(f, "Created \\/"),
            Sorting::ModifiedAsc => write!(f, "Created /\\"),
        }
    }
}
