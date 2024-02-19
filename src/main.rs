#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::egui;
use egui::*;
use std::{path, env};
use rfd::*;
use {
    grep_matcher::Matcher,
    grep_regex::RegexMatcher,
    grep_searcher::Searcher,
    grep_searcher::sinks::UTF8,
};

fn main()-> Result<(), eframe::Error> {
    //env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([960.0, 720.0]),
        ..Default::default()
    };
    eframe::run_native(
        "File Ripper",
        options,
        Box::new(|cc| {
            // This gives us image support:
            //egui_extras::install_image_loaders(&cc.egui_ctx);

            Box::<FileRipper>::default()
        }),
    )
}

struct SearchResult{
    result_string : String,
    result_full_path : String,
    result_search_score : u32,
}

impl Default for SearchResult{
    fn default() -> Self {
        SearchResult{
            result_string : String::from(""),
            result_full_path : String::from(""),
            result_search_score : 0,
        }
    }
}

struct FileRipper {
    name : String,
    cur_path : String, //want to use but borrowing is sucking Box<path::Path>,
    search_term : String, 
    search_results : Vec<SearchResult>,
}


impl Default for FileRipper {
    fn default() -> Self {
        Self {
            name : "FileRipper".to_string(),
            //TODO how can we compile time do C:\ for windows and ~ for Linux ??"
            cur_path : String::from(env::current_dir().expect("currend dir to be valid").to_str().expect("current_dir to be a str")),
            search_term : "".to_string(),
            search_results : vec![],
        }
    }
}

impl eframe::App for FileRipper{
    fn update(&mut self, ctx : &egui::Context, _frame : &mut eframe::Frame){
        egui::CentralPanel::default().show(ctx, |ui|{
            ui.horizontal(|ui|{
                //Left hand side, show files and directory items
                ui.vertical(|ui| {
                    let path_clone = &self.cur_path.clone();
                    let pathname = path::Path::new(path_clone);
                    ui.heading(path_clone);
                    match std::fs::read_dir(&self.cur_path) {
                        Ok(dirs) => {
                            //parent dir button
                            ui.horizontal(|ui|{
                               if ui.button("..").clicked(){
                                   self.cur_path = String::from(pathname.parent().unwrap().to_str().unwrap());
                               }
                            });

                            //buttons for all dirs and labels for files
                            for dir in dirs{
                                ui.horizontal(|ui|{
                                    let cur_file :String = String::from(dir.as_ref().unwrap().path().file_name().unwrap().to_str().unwrap());
                                    if dir.unwrap().metadata().unwrap().is_dir() {
                                        if ui.button(&cur_file).clicked(){
                                            self.cur_path = String::from(pathname.join(&cur_file).to_str().unwrap());
                                        }
                                    }
                                    else{
                                        ui.label(&cur_file);
                                    }
                                });
                            }
                        },
                        Err(err) => {
                            panic!("{}",err);
                        },
                    }
                });

                //Right hand side, search
                ui.vertical(|ui| {
                   ui.add_space(30.0);
                   ui.separator();
                   ui.heading("search...");
                   ui.text_edit_singleline(&mut self.search_term);
                   if 
                       //ui.text_edit_singleline(&mut self.search_term).lost_focus() && 
                       is_any_key_down(ctx)

                       {
                        //do ripgrep open dialog test

                        self.search_results = crawl_subdirs(self.cur_path.clone()); //@SPEED
                        for entry in &mut self.search_results{
                            
                            //let entry_clone = entry.clone(); //@SPEED
                            //let entry_score = 0;
                            /* The code to compute the levenshtein_distance makes the input run
                             * very slowly especially the longer the search term gets*/
                            let entry_score = levenshtein_distance(self.search_term.clone(), entry.result_string.to_owned()); //@SPEED
                            entry.result_search_score = entry_score;
                        }
                   }

                   for entry in &self.search_results{
                       if entry.result_search_score == entry.result_string.len().try_into().unwrap(){
                           ui.horizontal(|ui|{
                               ui.label(&entry.result_full_path);
                            });
                        }
                   }
                });

            });
        });
    }

}


pub fn crawl_subdirs(cur_dir : String) -> Vec<SearchResult>{
    let mut sub_dirs : Vec<SearchResult> = vec![];
    //lets go depth first down until we run out of directories to build our set
    //to compute distance to
    for dir in std::fs::read_dir(cur_dir).unwrap(){
        if dir.as_ref().unwrap().metadata().unwrap().is_dir(){
            sub_dirs.append(&mut crawl_subdirs(String::from(dir.unwrap().path().to_str().unwrap())));
        }
        else{
            //sub_dirs.push(String::from(dir.unwrap().path().to_str().unwrap()));
            let full_dir = String::from(dir.as_ref().unwrap().path().to_str().unwrap());
            let filename = String::from(dir.as_ref().unwrap().file_name().to_str().unwrap());
            sub_dirs.push(SearchResult{
                result_string : filename.clone(),
                result_full_path : full_dir,
                result_search_score : filename.len().try_into().unwrap(),
            });
        }
    }
    sub_dirs
}

pub fn levenshtein_distance(s1 : String, s2 : String) -> u32{
    if s2.len() > s1.len() {
        return levenshtein_distance(s2,s1);
    }

    if s1.len() == 0{
        return s2.len().try_into().unwrap();
    }
    if s2.len() == 0{
        return s1.len().try_into().unwrap();
    }

    let s2_len = s2.len() + 1;

    let mut previous;
    let mut temp;
    let mut current = vec![0;s2_len];

    for i in 1..s2_len{
        current[i] = i;
    }

    for(i,char_1) in s1.chars().enumerate(){
        previous = current[0];
        current[0] = i + 1;
        for(j,char_2) in s2.chars().enumerate(){
            temp = current[j + 1];
            current[j+1] = std::cmp::min(
                temp + 1, std::cmp::min(
                    current[j] + 1,
                    previous + if char_1 == char_2 {0} else {1}
                    )
                );
            previous = temp;
        }
    }
    current[s2_len-1].try_into().unwrap()
}

pub fn is_any_key_down(ctx : &egui::Context) -> bool {
    //TODO(ftommasi) is there a better way to do this ??
    ctx.input(|i| i. keys_down.len() > 0)
    /*
    if ctx.input(|i| 
                 i.key_pressed(Key::Q)
              || i.key_pressed(Key::W)
              || i.key_pressed(Key::E)
              || i.key_pressed(Key::R)
              || i.key_pressed(Key::T)
              || i.key_pressed(Key::Y)
              || i.key_pressed(Key::U)
              || i.key_pressed(Key::I)
              || i.key_pressed(Key::O)
              || i.key_pressed(Key::P)

              || i.key_pressed(Key::A)
              || i.key_pressed(Key::S)
              || i.key_pressed(Key::D)
              || i.key_pressed(Key::F)
              || i.key_pressed(Key::G)
              || i.key_pressed(Key::H)
              || i.key_pressed(Key::J)
              || i.key_pressed(Key::K)
              || i.key_pressed(Key::L)

              || i.key_pressed(Key::Z)
              || i.key_pressed(Key::X)
              || i.key_pressed(Key::C)
              || i.key_pressed(Key::V)
              || i.key_pressed(Key::B)
              || i.key_pressed(Key::N)
              || i.key_pressed(Key::M)

              || i.key_pressed(Key::ArrowDown)
              || i.key_pressed(Key::ArrowLeft)
              || i.key_pressed(Key::ArrowRight)
              || i.key_pressed(Key::ArrowUp)
              || i.key_pressed(Key::Escape)
              || i.key_pressed(Key::Tab)
              || i.key_pressed(Key::Backspace)
              || i.key_pressed(Key::Enter)
              || i.key_pressed(Key::Space)
              || i.key_pressed(Key::Insert)
              || i.key_pressed(Key::Delete)
              || i.key_pressed(Key::Home)
              || i.key_pressed(Key::End)
              || i.key_pressed(Key::PageUp)
              || i.key_pressed(Key::PageDown)
              || i.key_pressed(Key::Copy)
              || i.key_pressed(Key::Cut)
              || i.key_pressed(Key::Paste)
              || i.key_pressed(Key::Colon)
              || i.key_pressed(Key::Comma)
              || i.key_pressed(Key::Backslash)
              || i.key_pressed(Key::Slash)
              || i.key_pressed(Key::Pipe)
              || i.key_pressed(Key::Questionmark)
              || i.key_pressed(Key::OpenBracket)
              || i.key_pressed(Key::CloseBracket)
              || i.key_pressed(Key::Backtick)
              || i.key_pressed(Key::Minus)
              || i.key_pressed(Key::Period)
              || i.key_pressed(Key::Plus)
              || i.key_pressed(Key::Equals)
              || i.key_pressed(Key::Semicolon)
              || i.key_pressed(Key::Num0)
              || i.key_pressed(Key::Num1)
              || i.key_pressed(Key::Num2)
              || i.key_pressed(Key::Num3)
              || i.key_pressed(Key::Num4)
              || i.key_pressed(Key::Num5)
              || i.key_pressed(Key::Num6)
              || i.key_pressed(Key::Num7)
              || i.key_pressed(Key::Num8)
              || i.key_pressed(Key::Num9)
              ){
        return true;
    }
    false
        */
}
