/* Trumpet: A Mastodon client
 * Copyright (C) 2017-2018 Christopher Davis
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 */

extern crate kuchiki;
extern crate mammut;
extern crate toml;
extern crate webbrowser;
extern crate xdg;

use self::xdg::BaseDirectories;

use self::mammut::Data;
use self::mammut::{Mastodon, Registration, StatusBuilder};
use self::mammut::apps::{AppBuilder, Scopes};
use self::mammut::entities::status::Status;
use self::mammut::entities::account::Account;

use std::fs::File;

use std::io::prelude::*;
use std::io::{stdin, stdout};

use self::kuchiki::traits::TendrilSink;

pub struct App;

impl App {
    pub fn new() -> App {
        App
    }

    pub fn run(&self) {
        let mut masto = self.login_or_register();
        println!(
            "Logged on for @{}",
            masto.verify_credentials().unwrap().acct
        );

        // Loops until told to stop
        loop {
            let actions: [String; 7] = [
                "Make Status".to_string(),
                "View Local Timeline".to_string(),
                "View Home Timeline".to_string(),
                "Follow a user".to_string(),
                "View Instance Information".to_string(),
                "Log Out".to_string(),
                "Quit".to_string(),
            ];
            let choice = self.choose_actions(&actions);
            if choice == 1 {
                let stat = self.make_status();
                if masto.new_status(stat).is_err() {
                    println!("Could not send the status");
                } else {
                    println!("Status sent");
                }
            } else if choice == 2 {
                self.view_local_timeline(masto.clone());
            } else if choice == 3 {
                self.view_home_timeline(masto.clone());
            } else if choice == 4 {
                self.follow_users(masto.clone());
            } else if choice == 5 {
                self.view_instance_info(masto.clone());
            } else if choice == 6 {
                masto = self.login_or_register();
                println!(
                    "Logged on for @{}",
                    masto.verify_credentials().unwrap().acct
                );
            } else if choice == 7 {
                return;
            }
        }
    }

    fn choose_actions(&self, act: &[String]) -> u32 {
        println!("Choose an action: ");
        for (i, action) in act.iter().enumerate() {
            println!("[{}] {}", i + 1, action);
        }
        let input: u32 = self.input_loop() as u32;
        input
    }

    fn input_loop(&self) -> usize {
        let mut input_str = String::new();
        let input_num: usize;
        loop {
            print!("Type here: ");
            stdout().flush().unwrap();
            stdin().read_line(&mut input_str).unwrap();
            input_num = match input_str.trim().parse() {
                Ok(num) => num,
                Err(_) => {
                    println!("ERR: Not a valid number");
                    stdout().flush().unwrap();
                    continue;
                }
            };
            break;
        }
        input_num
    }

    // Code for following an account; Commented code is logic for when
    // following via mammut is functional
    fn follow(&self, account: &Account, _client: Mastodon) {
        println!(
            "Following is not yet implemented in Trumpet. Opening web browser for @{}",
            account.acct
        );
        webbrowser::open(&account.url).unwrap();
        // let id: u64 = account.id.parse().unwrap();

        // match _client.follow(id) {
        //     Ok(_) => println!("Now following @{}", account.acct),
        //     Err(e) => println!("Failed to folllow @{}\n{:?}", account.acct, e)
        // }
    }

    // Interactions for following
    fn follow_users(&self, client: Mastodon) {
        println!("Who would you like to follow?");
        let mut user_str = String::new();
        print!("Type here: ");
        stdout().flush().unwrap();
        stdin().read_line(&mut user_str).unwrap();

        let account_list = client
            .search_accounts(user_str.trim(), None, false)
            .unwrap()
            .initial_items;
        match account_list.len() {
            0 => {
                println!("Could not find any user with the given name.");
            }
            1 => {
                let acc_id = &account_list[0].id;
                if client.relationships(&[&acc_id]).unwrap().initial_items[0].following {
                    println!("You already follow this user.");
                } else {
                    self.follow(&account_list[0], client);
                }
            }
            _ => {
                println!("Choose which account to follow: ");
                for (i, account) in account_list.iter().enumerate() {
                    println!("[{}] @{}", i + 1, account.acct);
                }

                let input_num: usize = self.input_loop();

                let acc_id = &account_list[input_num - 1].id;
                if client.relationships(&[&acc_id]).unwrap().initial_items[0].following {
                    println!("You already follow this user.");
                } else {
                    self.follow(&account_list[input_num - 1], client);
                }
            }
        }
    }

    // Code for printing the timeline to the screen
    fn display_timeline(&self, timeline: &Vec<Status>) {
        for (i, status) in timeline.iter().enumerate() {
            let parser = kuchiki::parse_html();
            let node_ref = parser.one(&status.content[..]);
            let content_text = node_ref.text_contents();
            println!("{}. @{}: {}", i + 1, status.account.username, &content_text);
        }
    }

    // Views the local timeline; fails depending on instance
    fn view_local_timeline(&self, client: Mastodon) {
        let timeline = match client.get_public_timeline(true) {
            Ok(timeline) => timeline,
            Err(e) => {
                println!("Could not view timeline");
                println!("{:?}", e);
                return;
            }
        };
        self.display_timeline(&timeline);
    }

    // Views the home timeline; fails depending on instance
    fn view_home_timeline(&self, client: Mastodon) {
        let timeline = match client.get_home_timeline() {
            Ok(timeline) => timeline,
            Err(e) => {
                println!("Could not view timeline");
                println!("{:?}", e);
                return;
            }
        };
        self.display_timeline(&timeline.initial_items);
    }

    // Exactly as it says on the tin
    fn view_instance_info(&self, client: Mastodon) {
        println!("{} via Trumpet", client.instance().unwrap().uri);
        println!("Description: {}", client.instance().unwrap().description);
        println!("Email: {}", client.instance().unwrap().email);
    }

    // This returns a StatusBuilder based on the options
    // chosen. The StatusBuilder is then passed into
    // mammut's status function.
    fn make_status(&self) -> StatusBuilder {
        // The string sent by the status
        let mut status_str = String::new();
        print!("Status text: ");
        stdout().flush().unwrap();
        stdin().read_line(&mut status_str).unwrap();

        let status_str = status_str.trim().to_string();
        println!("Spoiler text: ");
        let yes_no: [String; 2] = ["Yes".to_owned(), "No".to_owned()];
        let choice = self.choose_actions(&yes_no);
        let spoiler_text = if choice == 1 {
            print!("Text here: ");
            let mut spoiler_str = String::new();
            stdout().flush().unwrap();
            stdin().read_line(&mut spoiler_str).unwrap();
            Some(spoiler_str)
        } else {
            None
        };

        //TODO: Allow user to fill fields of StatusBuilder
        StatusBuilder {
            status: status_str,
            in_reply_to_id: None,
            media_ids: None,
            sensitive: None,
            spoiler_text: spoiler_text,
            visibility: None,
        }
    }

    // Loads login data from file.
    fn load_conf(&self, mut file: File) -> Mastodon {
        let mut conf = String::new();
        file.read_to_string(&mut conf).unwrap();
        let data: Data = toml::from_str(&conf).unwrap();
        Mastodon::from_data(data)
    }

    // Exactly as it says on the tin
    fn login_or_register(&self) -> Mastodon {
        let xdg_dir = BaseDirectories::with_prefix("Trumpet").unwrap();
        let data_files = xdg_dir.list_data_files("");

        if data_files.len() == 0 {
            return self.register();
        }

        let actions = [
            "Login to existing instance".to_string(),
            "Register new instance".to_string(),
        ];

        let mut choice = self.choose_actions(&actions);
        loop {
            if choice == 1 {
                return self.login();
            } else if choice == 2 {
                return self.register();
            } else {
                choice = self.choose_actions(&actions);
            }
        }
    }

    fn login(&self) -> Mastodon {
        let xdg_dir = BaseDirectories::with_prefix("Trumpet").unwrap();
        let data_files = xdg_dir.list_data_files("");

        println!("Choose file: ");
        for (i, data_file) in data_files.iter().enumerate() {
            println!(
                "[{}] Load {}",
                i + 1,
                data_file.file_name().unwrap().to_str().unwrap()
            );
        }
        let mut input: usize;
        loop {
            input = self.input_loop();
            if input > data_files.len() {
                println!("ERR: Input is greater than number of existing files");
                continue;
            }
            break;
        }
        let masto = match File::open(&data_files[input - 1]) {
            Ok(file) => self.load_conf(file),
            Err(_) => self.register(),
        };

        masto
    }

    fn register(&self) -> Mastodon {
        let masto_app = AppBuilder {
            client_name: "Trumpet",
            redirect_uris: "urn:ietf:wg:oauth:2.0:oob",
            scopes: Scopes::All,
            website: Some("https://github.com/BrainBlasted/Trumpet"),
        };

        let mut masto_instance_url = String::new();
        print!("Instance URL: https://");
        stdout().flush().unwrap();
        stdin().read_line(&mut masto_instance_url).unwrap();

        masto_instance_url = format!("https://{}/", masto_instance_url.trim().to_string());

        let mut regist = Registration::new(masto_instance_url.clone());

        // Checks if the registration process can continue with current
        // url. If not, restarts the regostration process.
        if regist.register(masto_app).is_err() {
            println!("Could not autheticate with the Mastodon instance.");
            println!("Check that you entered the url of an existing instance.");
            return self.register();
        }

        let auth_url = regist.authorise().unwrap();

        let mut auth_code = String::new();

        println!("Opening {}", masto_instance_url);

        webbrowser::open(&auth_url).expect("Could not open web browser");

        print!("Auth Code from browser: ");
        stdout().flush().unwrap();
        stdin()
            .read_line(&mut auth_code)
            .expect("Could not read auth code");

        let auth_code = auth_code.trim().to_string();

        let masto = match regist.create_access_token(auth_code) {
            Ok(reg) => reg,
            Err(_) => {
                println!("Could not create an acces token. Double check that you entered your code correctly.");
                self.register()
            }
        };

        // Write registration data to config file
        let toml = toml::to_string(&*masto).unwrap();
        let xdg_dir = BaseDirectories::with_prefix("Trumpet").expect("Could not find prefix");

        let username = match masto.verify_credentials() {
            Ok(accnt) => accnt.username,
            Err(e) => {
                panic!("Failed verifying credentials with {}", e);
            }
        };
        let instance_uri = match masto.instance() {
            Ok(inst) => inst.uri,
            Err(e) => {
                panic!("Failed receiving instance data with {}", e);
            }
        };
        let data_file_str = format!("{}@{}", username, instance_uri);
        let data_file_path = xdg_dir
            .place_data_file(data_file_str)
            .expect("Could not place data file");
        let mut file = match File::open(data_file_path.clone()) {
            Ok(file) => file,
            Err(_) => File::create(data_file_path).expect("Could not create data file"),
        };
        file.write_all(toml.as_bytes()).unwrap();

        masto
    }
}
