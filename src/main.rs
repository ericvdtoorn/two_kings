#![allow(unused)]
use eframe::egui;
use egui::{pos2, Color32, FontId, Rect, RichText, Sense, TextBuffer};
use serde::{
    de::{Error, Visitor},
    Deserialize, Serialize,
};
use std::{
    collections::{BTreeMap, HashMap},
    fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
//#[serde(format = "lowercase")]
enum Color {
    Pink,
    Blue,
    LightBlue,
    Yellow,
    Purple,
    LightGreen,
    DarkGreen,
    Salmon,
    Teal,
    Orange,
}

impl Color {
    fn to_color32(&self) -> Color32 {
        match self {
            Color::Pink => Color32::from_rgb(255, 192, 203),
            Color::Blue => Color32::from_rgb(100, 149, 237),
            Color::LightBlue => Color32::from_rgb(173, 216, 230),
            Color::Yellow => Color32::from_rgb(255, 255, 0),
            Color::Purple => Color32::from_rgb(128, 0, 128),
            Color::LightGreen => Color32::from_rgb(144, 238, 144),
            Color::DarkGreen => Color32::DARK_GREEN.clone(),
            Color::Salmon => Color32::from_rgb(250, 128, 114),
            Color::Teal => Color32::from_rgb(0, 128, 128),
            Color::Orange => Color32::from_rgb(255, 165, 0),
        }
    }
}
#[derive(Debug, Clone, Copy)]
struct Position {
    x: usize,
    y: usize,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd)]
enum ClickState {
    Empty,
    Eliminated,
    King(bool),
}
const MAX_KINGS: usize = 2;
struct Grid {
    colors: Vec<Vec<Color>>,
    state: Vec<Vec<ClickState>>,
}

fn touching_neighbors(x: usize, y: usize, width: usize, height: usize) -> Vec<Position> {
    let mut res = vec![];
    for i in [-1, 0, 1] {
        let new_x = (x as i64) + i;
        if new_x < 0 {
            continue;
        }
        let new_x = new_x as usize;
        if new_x >= width {
            continue;
        }

        for j in [-1, 0, 1] {
            if i == 0 && j == 0 {
                continue;
            }
            let new_y = (y as i64) + j;
            if new_y < 0 {
                continue;
            }
            let new_y = new_y as usize;
            if new_y >= height {
                continue;
            }
            res.push(Position { x: new_x, y: new_y });
        }
    }
    res
}

impl Grid {
    fn load_from_json(path: impl Into<PathBuf>) -> Result<Self, Box<dyn std::error::Error>> {
        let json_str = fs::read_to_string(path.into())?;
        let colors: Vec<Vec<Color>> = serde_json::from_str(&json_str)?;
        let state = vec![ClickState::Empty; colors[0].len()];
        let mut state = vec![state; colors.len()];
        state[5][5] = ClickState::King(false);
        state[6][5] = ClickState::Eliminated;
        Ok(Self { colors, state })
    }

    fn save_to_json(&self) -> Result<(), Box<dyn std::error::Error>> {
        let json_str = serde_json::to_string_pretty(&self.colors)?;
        fs::write("grid_colors.json", json_str)?;
        Ok(())
    }

    fn check_state(&mut self) -> bool {
        let height = self.colors.len();
        let width = self.colors[0].len();
        let mut new_state = self.state.clone();
        // set all the kings back to false
        new_state.iter_mut().for_each(|row| {
            row.iter_mut()
                .filter(|cell| matches!(cell, ClickState::King(true)))
                .for_each(|cell| {
                    *cell = ClickState::King(false);
                })
        });

        let mut kings_in_ys = vec![0; height];
        let mut kings_in_xs = vec![0; width];
        let mut kings_in_colors = HashMap::new();
        for y in 0..height {
            for x in 0..width {
                let s = self.state[y][x];
                if matches!(s, ClickState::King(_)) {
                    kings_in_ys[y] += 1;
                    kings_in_xs[x] += 1;
                    let color = self.colors[y][x];
                    kings_in_colors
                        .entry(color)
                        .or_insert(vec![])
                        .push(Position { x, y });

                    if touching_neighbors(x, y, width, height)
                        .iter()
                        .any(|c| matches!(self.state[c.y][c.x], ClickState::King(_)))
                    {
                        new_state[y][x] = ClickState::King(true);

                        for c in touching_neighbors(x, y, width, height).iter() {
                            if matches!(self.state[c.y][c.x], ClickState::King(_)) {
                                new_state[c.y][c.x] = ClickState::King(true);
                            }
                        }
                    }
                }
            }
        }
        // check the rows
        for y in 0..height {
            if kings_in_ys[y] > MAX_KINGS {
                // change all the kings to InvalidKing
                for x in 0..width {
                    new_state[y][x] = if matches!(self.state[y][x], ClickState::King(_)) {
                        ClickState::King(true)
                    } else {
                        self.state[y][x]
                    }
                }
            }
        }

        // check the columns
        for x in 0..width {
            if kings_in_xs[x] > MAX_KINGS {
                // change all the kings to InvalidKing
                for y in 0..height {
                    new_state[y][x] = if matches!(self.state[y][x], ClickState::King(_)) {
                        ClickState::King(true)
                    } else {
                        self.state[y][x]
                    }
                }
            }
        }

        // check the colors
        for (color, kings) in kings_in_colors.iter() {
            if kings.len() > MAX_KINGS {
                for king_pos in kings {
                    new_state[king_pos.y][king_pos.x] = ClickState::King(true);
                }
            }
        }
        self.state = new_state;
        if kings_in_xs.iter().sum::<usize>() == MAX_KINGS * width {
            return true;
        }
        return false;
    }
}

impl eframe::App for Grid {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let available_size = ui.available_size();
            let grid_size = available_size.x.min(available_size.y);
            let cell_size = grid_size / 10.0;

            ui.label("Enable helper??");
            //ui.put(max_rect, widget)
            //let rect = ui.add_sized(grid_size, )
            let (response, painter) =
                ui.allocate_painter(egui::vec2(grid_size, grid_size), egui::Sense::click());
            let rect = response.rect;

            let finished = self.check_state();
            if finished {
                ui.label(
                    RichText::new("Game Complete!")
                        .color(Color32::DARK_GREEN)
                        .font(FontId::proportional(40.0)),
                );
            }
            // Draw the grid
            for (y, row) in self.colors.iter().enumerate() {
                for (x, &color) in row.iter().enumerate() {
                    let cell_rect = egui::Rect::from_min_size(
                        rect.min + egui::vec2(x as f32 * cell_size, y as f32 * cell_size),
                        egui::vec2(cell_size, cell_size),
                    );

                    let r = ui.allocate_rect(cell_rect, Sense::click());
                    let current_state = self.state[y][x];

                    use ClickState::*;
                    let new_state = match (current_state, r.clicked()) {
                        (_, false) => current_state,
                        (Empty, true) => Eliminated,
                        (Eliminated, true) => King(false),
                        (King(_), true) => Empty,
                    };

                    // Draw initial cell
                    painter.rect_filled(cell_rect, 0.0, color.to_color32());
                    painter.rect_stroke(cell_rect, 0.0, egui::Stroke::new(1.0, Color32::BLACK));
                    self.state[y][x] = new_state;
                    match new_state {
                        ClickState::Empty => {}

                        ClickState::Eliminated => {
                            painter.circle_filled(cell_rect.center(), 2.1, Color32::BLACK);
                        }
                        ClickState::King(false) => {
                            //painter.circle_filled(cell_rect.center(), 5.1, Color32::GREEN);
                            egui::Image::new(egui::include_image!("../static/king.png"))
                                //.rounding(5.0)
                                .tint(egui::Color32::WHITE)
                                .paint_at(ui, cell_rect);
                        }

                        ClickState::King(true) => {
                            painter.circle_filled(cell_rect.center(), 5.1, Color32::RED);
                            //egui::Image::new(egui::include_image!("../static/king.png"))
                            //    //.rounding(5.0)
                            //    .tint(egui::Color32::RED)
                            //    .paint_at(ui, cell_rect);
                        }
                    }
                    // Draw cell
                }
            }
        });
    }
}

fn main() -> Result<(), eframe::Error> {
    env_logger::init();
    let options = eframe::NativeOptions {
        //initial_window_size: Some(egui::vec2(500.0, 500.0)),
        viewport: egui::ViewportBuilder::default().with_inner_size([320.0, 240.0]),
        ..Default::default()
    };
    let grid = Grid::load_from_json("static/grid.json").expect("Could not load grid from file");

    eframe::run_native("Color Grid", options, Box::new(|_cc| Ok(Box::new(grid))))
}
