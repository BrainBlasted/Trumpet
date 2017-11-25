// Trumpet: A Mastodon client
// Copyright (C) 2017 Christopher Davis
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

extern crate gtk;
extern crate xdg;
extern crate toml;
extern crate mammut;
extern crate webbrowser;

use self::gtk::prelude::*;
use self::gtk::{Button, Window, WindowType};

use self::xdg::BaseDirectories;

use self::mammut::Data;
use self::mammut::{Registration, StatusBuilder, Mastodon};
use self::mammut::apps::{AppBuilder, Scope};

use std::fs::File;
use std::fs::remove_file;

use std::io;
use std::io::prelude::*;
use std::io::{stdin, stdout};

pub struct App {

}

impl App {
    pub fn run(&self) {

        // Checking if GTK properly initializes.
        if gtk::init().is_err() {
            println!("Failed to initialize GTK");
            return;
        }

        let main_window = Window::new(WindowType::Toplevel);

        let xdg_dir = BaseDirectories::with_prefix("Trumpet").unwrap();

        // Checks if path to trumpet-data already exists; if not,
        // assigns to a new path.
        let data_file_path = match xdg_dir.find_data_file("trumpet-data.toml") {
            Some(path) => path,
            None => xdg_dir.place_data_file("trumpet-data.toml").unwrap(),
        };

        // If the data file can be opened load the configuration
        // from the file. Otherwise, register new data.
        let mut masto = match File::open(data_file_path.clone()) {
            Ok(file) => self.load_conf(file),
            Err(_) => self.register(),
        };

        // Loops until told to stop
        loop {
            let actions: [String; 4] = [
                "Make Status".to_string(),
                "View Timeline".to_string(),
                "Log Out".to_string(),
                "Quit".to_string()
            ];
            let choice = self.choose_actions(&actions);
            if choice == 1 {
                let stat = self.make_status();
                masto.new_status(stat).unwrap();
            } else if choice == 2 {
                self.view_timeline(masto.clone());
            } else if choice == 3 {
                remove_file(data_file_path.clone()).unwrap();
                masto = self.register();
            } else if choice == 4 {
                return;
            }
        }
    }

    pub fn choose_actions(&self, act: &[String]) -> i32 {
        println!("Choose an action: ");
        for (i, action) in act.iter().enumerate() {
            println!("[{}] {}", i+1, action);
        }
        let mut input_str;
        let input: i32;
        loop {
            input_str = String::new();
            print!("Type here: ");
            stdout().flush().unwrap();
            match stdin().read_line(&mut input_str) {
                Ok(string) => string,
                Err(_) => {
                    println!("ERR: Could not read line");
                    continue;
                }
            };
            input = match input_str.trim().parse() {
                Ok(num) => num,
                Err(_) => {
                    println!("ERR: Could not parse your input");
                    continue;
                },
            };
            break;
        }
        input
    }

    pub fn view_timeline(&self, masto: Mastodon) {
        let timeline = masto.get_public_timeline(true).unwrap();
        for (i, status) in timeline.iter().enumerate() {
            println!("{}. @{}: {}", i+1, status.account.username, status.content);
        }
    }

    pub fn make_status(&self) -> StatusBuilder {
        // The string sent by the status
        let mut status_str = String::new();
        print!("Status text: ");
        stdout().flush().unwrap();
        stdin().read_line(&mut status_str).unwrap();

        let status_str = status_str.trim().to_string();

        //TODO: Allow user to fill fields of StatusBuilder
        StatusBuilder {
            status: status_str,
            in_reply_to_id: None,
            media_ids: None,
            sensitive: None,
            spoiler_text: None,
            visibility: None,
        }

    }

    pub fn load_conf(&self, mut file: File) -> Mastodon {
        let mut conf = String::new();
        file.read_to_string(&mut conf).unwrap();
        let data: Data = toml::from_str(&conf).unwrap();
        Mastodon::from_data(data)
    }

    pub fn register(&self) -> Mastodon {

        let masto_app = AppBuilder {
            client_name: "Trumpet",
            redirect_uris: "urn:ietf:wg:oauth:2.0:oob",
            scopes: Scope::All,
            website: None,
        };

        let mut masto_instance_url = String::new();
        loop {
            print!("Instance URL: https://");
            io::stdout().flush().unwrap();
            match io::stdin().read_line(&mut masto_instance_url) {
                Ok(string) => string,
                Err(_) => {
                    println!("ERR: Could not read input");
                    continue;
                },
            };
            break;
        }

        masto_instance_url = format!("https://{}/", masto_instance_url.trim().to_string());

        let mut regist = Registration::new(masto_instance_url.clone());

        regist.register(masto_app).expect("Not a valid mastodon instance");

        let auth_url = regist.authorise().unwrap();

        let mut auth_code = String::new();

        println!("Opening {}", masto_instance_url);

        webbrowser::open(&auth_url)
            .expect("Could not open web browser");

        print!("Auth Code from browser: ");
        io::stdout().flush().unwrap();
        io::stdin().read_line(&mut auth_code)
            .expect("Could not read auth code");

        let auth_code_str = auth_code.trim().to_string();

        let masto = regist.create_access_token(auth_code_str)
            .expect("Could not create access token. Did you enter the code correctly?");

        // Write registration data to config file
        let toml = toml::to_string(&*masto).unwrap();
        let xdg_dir = BaseDirectories::with_prefix("Trumpet").expect("Could not find prefix");
        let data_file_path = xdg_dir.place_data_file("trumpet-data.toml")
            .expect("Could not place data file");
        let mut file = match File::open(data_file_path.clone()) {
            Ok(file) => file,
            Err(_) => File::create(data_file_path).expect("Could not create data file"),
        };
        file.write_all(toml.as_bytes()).unwrap();

        masto
    }
}
