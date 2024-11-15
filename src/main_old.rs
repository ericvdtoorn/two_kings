#![allow(unused)]
use std::str::FromStr;

use eframe::egui;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([320.0, 240.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Two Kings",
        options,
        Box::new(|cc| {
            // This gives us image support:
            egui_extras::install_image_loaders(&cc.egui_ctx);

            Ok(Box::<MyApp>::default())
        }),
    )
}

struct MyApp {
    game: KingGame,
    name: String,
    age: u32,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            name: "Arthur".to_owned(),
            game: KingGame::parse(EXAMPLE_GAME_JSON).unwrap(),
            age: 42,
        }
    }
}

impl KingGame {
    fn parse(s: &str) -> anyhow::Result<Self> {
        let mut colorings: Vec<Vec<String>> = serde_json::from_str(s)?;
        let colorings = colorings
            .into_iter()
            .map(|row| {
                row.into_iter()
                    .map(|value| value.parse().unwrap())
                    .collect()
            })
            .collect();
        Ok(Self {
            placed_kings: vec![],
            colorings,
        })
    }
}

const EXAMPLE_GAME_JSON: &str = r#"[
  ["pink", "pink", "pink", "pink", "pink", "pink", "blue", "blue", "blue", "blue"],
  ["lightblue", "pink", "pink", "yellow", "yellow", "blue", "blue", "blue", "blue", "blue"],
  ["lightblue", "pink", "pink", "yellow", "yellow", "purple", "blue", "blue", "blue", "lightgreen"],
  ["lightblue", "pink", "pink", "yellow", "yellow", "purple", "purple", "purple", "blue", "lightgreen"],
  ["lightblue", "pink", "yellow", "yellow", "yellow", "yellow", "purple", "purple", "blue", "lightgreen"],
  ["lightblue", "yellow", "yellow", "salmon", "purple", "purple", "purple", "purple", "teal", "purple"],
  ["yellow", "yellow", "yellow", "salmon", "salmon", "lightgreen", "lightgreen", "purple", "teal", "purple"],
  ["yellow", "yellow", "yellow", "salmon", "salmon", "lightgreen", "lightgreen", "purple", "teal", "purple"],
  ["salmon", "salmon", "salmon", "salmon", "salmon", "lightgreen", "lightgreen", "teal", "teal", "teal"],
  ["salmon", "salmon", "salmon", "salmon", "orange", "orange", "teal", "teal", "teal", "teal"]
]"#;

#[derive(Debug, Clone, Copy)]
enum Color {
    Pink,
    Blue,
    LightBlue,
    Yellow,
    LightGreen,
    Purple,
    Teal,
    Salmon,
    Orange,
}

impl FromStr for Color {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use Color::*;
        Ok(match s {
            "pink" => Pink,
            "blue" => Blue,
            "lighblue" => LightBlue,
            //"blue" => Blue,
            //"blue" => Blue,
            //"blue" => Blue,
            _ => Orange,
            //_ => return Err(format!("{} is not defined as a color", s)),
        })
    }
}

#[derive(Debug, Clone, Copy)]
struct Position {
    x: usize,
    y: usize,
}

impl Position {
    fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }
}

#[derive(Debug)]
struct KingGame {
    placed_kings: Vec<Position>,
    colorings: Vec<Vec<Color>>,
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("A Two Kings game solver");
            ui.horizontal(|ui| {
                let name_label = ui.label("Your name: ");
                ui.text_edit_singleline(&mut self.name)
                    .labelled_by(name_label.id);
            });
            ui.add(egui::Slider::new(&mut self.age, 0..=120).text("age"));
            if ui.button("Increment").clicked() {
                self.age += 1;
            }
            ui.label(format!("Hello '{}', age {}", self.name, self.age));

            ui.image(egui::include_image!("../static/king.png"));
        });
    }
}
