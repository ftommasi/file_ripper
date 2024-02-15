#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::egui;
use egui::*;
use std::path;
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

struct FileRipper {
    name : String,
    cur_path : String, //want to use but borrowing is sucking Box<path::Path>,
    search_term : String, 
    search_results : Vec<String>,
}


impl Default for FileRipper {
    fn default() -> Self {
        Self {
            name : "FileRipper".to_string(),
            cur_path : "C:/".to_string(),
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
                   ui.heading("serach...");
                   if ui.text_edit_singleline(&mut self.search_term).lost_focus() && ctx.input(|i| i.key_pressed(Key::Enter)){
                        //do ripgrep open dialog test

                        self.search_results = crawl_subdirs(self.cur_path.clone()); //@SPEED
                        
                        let mut min_lev_score = 99999; //What should be the initial value?
                        for entry in &self.search_results{
                            let entry_clone = entry.clone(); //@SPEED
                            let entry_score = levenshtein_distance(self.search_term.clone(), entry.to_owned()); //@SPEED
                            if min_lev_score < entry_score{
                                min_lev_score = entry_score;
                                println!("word: {} score: {}\n",self.search_term.clone(),entry_score);
                            }
                        }
                   }

                   for entry in &self.search_results{
                       let entry_clone = entry.clone(); //@SPEED
                       ui.horizontal(|ui|{
                           ui.label(entry_clone);
                       });
                   }
                });

            });
        });
    }

}

pub fn crawl_subdirs(cur_dir : String) -> Vec<String>{
    let mut sub_dirs : Vec<String> = vec![];
    //lets go depth first down until we run out of directories to build our set
    //to compute distance to
    for dir in std::fs::read_dir(cur_dir).unwrap(){
        if dir.as_ref().unwrap().metadata().unwrap().is_dir(){
            sub_dirs.append(&mut crawl_subdirs(String::from(dir.unwrap().path().to_str().unwrap())));
        }
        else{
            sub_dirs.push(String::from(dir.unwrap().path().to_str().unwrap()));
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
