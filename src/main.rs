extern crate mammut;
extern crate xdg;
extern crate toml;
extern crate webbrowser;

use xdg::BaseDirectories;

use mammut::Data;
use mammut::{Registration, StatusBuilder, Mastodon};
use mammut::apps::{AppBuilder, Scope};

use std::fs::File;

use std::io;
use std::io::prelude::*;
use std::io::{stdin, stdout};

fn main() {
    let xdg_dir = BaseDirectories::with_prefix("Trumpet").unwrap();

    let data_file_path = match xdg_dir.find_data_file("trumpet-data.toml") {
        Some(path) => path,
        None => xdg_dir.place_data_file("trumpet-data.toml").unwrap(),
    };

    let mut masto = match File::open(data_file_path.clone()) {
        Ok(file) => load_conf(file),
        Err(_) => register(),
    };

    loop {
        let actions: [String; 3] = [
            "Make Status".to_string(),
            "Log Out".to_string(),
            "Quit".to_string()
        ];
        let choice = choose_actions(&actions);
        if choice == 1 {
            let stat = make_status(); 
            masto.new_status(stat).unwrap();
        } else if choice == 2 {
            std::fs::remove_file(data_file_path.clone()).unwrap();
            masto = register();
        } else if choice == 3 {
            return;
        }
    }
}

fn choose_actions(act: &[String]) -> i32 {
    println!("Choose an action: ");
    let mut i = 0;
    while i < act.len() {
        println!("[{}] {}", i + 1, act[i]);
        i+=1;
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

fn make_status() -> StatusBuilder {
    // The string sent by the status
    let mut status_str = String::new();
    print!("Status text: ");
    stdout().flush().unwrap();
    stdin().read_line(&mut status_str).unwrap();

    let status_str = status_str.trim().to_string();

    StatusBuilder {
        status: status_str,
        in_reply_to_id: None,
        media_ids: None,
        sensitive: None,
        spoiler_text: None,
        visibility: None,
    }

}

fn load_conf(mut file: File) -> Mastodon {
    let mut conf = String::new();
    file.read_to_string(&mut conf).unwrap();
    let data: Data = toml::from_str(&conf).unwrap();
    Mastodon::from_data(data)
}

fn register() -> Mastodon {

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

    let auth_code_str: String = auth_code.trim().to_string();

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
