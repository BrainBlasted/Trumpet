extern crate mammut;
extern crate webbrowser;

use mammut::{Registration, StatusBuilder, Mastodon};
use mammut::apps::{AppBuilder, Scope};

use std::io;
use std::io::prelude::*;
use std::io::{stdin, stdout};

fn main() {
    let masto = register();

//    let stat = make_status(); 
//    masto.new_status(stat).unwrap();
    loop {
        let choice = choose_actions();
        if choice == 1 {
            let stat = make_status(); 
            masto.new_status(stat).unwrap();
        } else if choice == 2 {
            return;
        }
    }
}

fn choose_actions() -> i32 {
    println!("Choose an action: ");
    println!("[1] Make status");
    println!("[2] Log Out");
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

fn register() -> Mastodon {

    let masto_app = AppBuilder {
        client_name: "masto_mammut_cli",
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

    println!("{} was the URL given", masto_instance_url);

    let mut regist = Registration::new(masto_instance_url);

    regist.register(masto_app).unwrap();

    let auth_url = regist.authorise().unwrap();

    let mut auth_code = String::new();

    match webbrowser::open(&auth_url) {
        Ok(_) => {
            loop {
                print!("Auth Code from browser: ");
                io::stdout().flush().unwrap();
                match io::stdin().read_line(&mut auth_code) {
                    Ok(string) => string,
                    Err(_) => {
                        println!("Could not read auth code");
                        continue;
                    }
                };
                break;
            }
        },
        Err(_) => {
            println!("Could not open web browser.");
        },
    }

    let auth_code_str: String = auth_code.trim().to_string();

    let masto = regist.create_access_token(auth_code_str).unwrap();

    masto
}
