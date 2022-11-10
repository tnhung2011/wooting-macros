use std::{fs, result, thread, time};
use std::borrow::{Borrow, BorrowMut};
use std::collections::HashMap;
use std::fmt::{format, Formatter};
use std::fs::File;
use std::hash::Hash;
use std::ptr::hash;
use std::str::{Bytes, FromStr};
use std::sync::mpsc::{channel, SendError};
use std::sync::RwLock;
use std::time::Duration;

use lazy_static::lazy_static;
use rdev::{Button, Event, EventType, grab, Key, listen, simulate, SimulateError};
use serde::Serialize;
use tauri::{Config, State};

use crate::{APPLICATION_STATE, ApplicationConfig, hid_table};
use crate::hid_table::*;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum MacroType {
    Single,
    Repeating,
    OnHold,
    MultiLevel,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Hash, Eq)]
pub enum KeyType {
    DownUp,
    Down,
    Up,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Hash, Eq)]
pub struct KeyPress {
    pub keypress: u32,
    pub press_duration: Delay,
    pub keytype: KeyType,
}
//
// impl KeyboardKeypress {
//     fn execute_key_up(&self, key_to_release: &rdev::Key) {
//         send();
//     }
//
//     fn execute_key_down(&self) {}
// }

///Delay for the sequence
pub type Delay = u64;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum ActionEventType {
    KeyPressEvent { data: KeyPress },
    //KeyON
    //KeyOFF
    //SystemEvent { action: Action },
    PhillipsHueCommand {},
    OBS {},
    DiscordCommand {},
    //IKEADesk
    //MouseMovement
    UnicodeDirect {},
    Delay { data: Delay },
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum TriggerEventType {
    KeyPressEvent { data: Vec<KeyPress> },
}

#[derive(Debug, Clone)]
pub struct EventList(Vec<rdev::Key>);

#[derive(Debug, Clone)]
pub struct Action {
    pub action: char,
    pub press_wait_delay_after: Delay,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Macro {
    pub name: String,
    pub sequence: Vec<ActionEventType>,
    pub macro_type: MacroType,
    pub trigger: TriggerEventType,
    pub active: bool,
}

impl Macro {
    fn new() -> Macro {
        Macro {
            name: "".to_string(),
            sequence: vec![],
            macro_type: MacroType::Single,
            trigger: TriggerEventType::KeyPressEvent { data: vec![] },
            active: false,
        }
    }
}

#[tauri::command]
/// Gets the application config from the current state and sends to frontend.
/// The state gets it from the config file at bootup.
pub fn get_config(state: tauri::State<MacroDataState>) -> ApplicationConfig {
    let config_data_state = state.config.read().unwrap();
    config_data_state.clone()
}

#[tauri::command]
/// Gets the application config from the current state and sends to frontend.
/// The state gets it from the config file at bootup.
pub fn set_config(state: tauri::State<MacroDataState>, config: ApplicationConfig) {
    let mut tauri_state = state.config.write().unwrap();
    *tauri_state = config.clone();
    tauri_state.export_data();

    let mut app_state = APPLICATION_STATE.config.write().unwrap();
    *app_state = config;
}

#[tauri::command]
/// Gets the macro data from current state and sends to frontend.
/// The state gets it from the config file at bootup.
pub fn get_macros(state: tauri::State<MacroDataState>) -> MacroData {
    let macro_data_state = state.data.read().unwrap();
    macro_data_state.clone()
}

#[tauri::command]
/// Sets the configuration from frontend and updates the state for everything on backend.
pub fn set_macros(state: tauri::State<MacroDataState>, frontend_data: MacroData) {
    let mut tauri_state = state.data.write().unwrap();
    *tauri_state = frontend_data.clone();
    tauri_state.export_data();

    let mut app_state = APPLICATION_STATE.data.write().unwrap();
    *app_state = frontend_data;
}

/// Function for a manual write of config changes from the backend side. Just a test.
/// Not meant to be used.
pub fn set_data_write_manually_backend(frontend_data: MacroData) {
    let mut app_state = APPLICATION_STATE.data.write().unwrap();
    *app_state = frontend_data.clone();
    app_state.clone().export_data();
}

//
// fn check_key(incoming_key: &Vec<rdev::Key>) {
//     let app_state = APPLICATION_STATE.data.read().unwrap();
//
//     for collections in &app_state.data {
//         if collections.active == true {
//             for macros in &collections.macros {
//                 if macros.active == true {
//                     match &macros.trigger {
//                         TriggerEventType::KeyPressEvent { data: trigger } => {
//                             for i in trigger {
//                                 if SCANCODE_MAP[&i.keypress] == *incoming_key {
//                                     println!(
//                                         "FOUND THE TRIGGER, WOULD EXECUTE MACRO: {}",
//                                         macros.name
//                                     )
//                                 }
//                             }
//                         }
//                     }
//                 }
//             }
//         }
//     }
// }

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct MacroDataState {
    pub data: RwLock<MacroData>,
    pub config: RwLock<ApplicationConfig>,
}

impl MacroDataState {
    pub fn new() -> Self {
        MacroDataState {
            data: RwLock::from(MacroData::read_data()),
            config: RwLock::from(ApplicationConfig::read_data()),
        }
    }
}

// ///Hash list
// #[derive(Debug, Clone, Hash, Eq, PartialEq)]
// pub struct TriggerHash<'a> {
//     trigger_table: HashMap<Vec<rdev::Key>, &'a Macro>,
// }

type TriggersExtracted<'a> = Vec<(Vec<u32>, Vec<rdev::Key>, &'a Macro)>;

type Collections = Vec<Collection>;

///MacroData is the main data structure that contains all macro data.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MacroData {
    pub data: Collections,
}

impl MacroData {
    /// This exports data for the frontend to process it.
    /// Basically sends the entire struct to the frontend
    pub fn export_data(&self) {
        std::fs::write(
            "../data_json.json",
            serde_json::to_string_pretty(&self).unwrap(),
        )
            .unwrap();
    }

    // /// Imports data from the frontend (when updated) to update the background data structure
    // /// This overwrites the datastructure
    // pub fn import_data(&mut self, input: MacroData) -> TriggerHash {
    //     *self = input;
    //     self.export_data();
    //     self.extract_triggers()
    // }

    /*
        /// Extracts the data. This is a helper function for now.
        fn extract_triggers(&self) -> TriggersExtracted {
            //vector of keys
            let mut output: TriggersExtracted = vec![];
            // let mut add_keys: Vec<u32> = vec![];
            // let mut add_keys_converted: Vec<rdev::Key> = vec![];
            // let mut macro_to_add: &Macro = &Macro::new();


            let mut tuple: TriggersExtracted = vec![];
            for groups in &self.data {
                for macros in &groups.macros{
                    let mut macro_to_add: Macro = Macro{
                        name: "".to_string(),
                        sequence: vec![],
                        macro_type: MacroType::Single,
                        trigger: TriggerEventType::KeyPressEvent { data: vec![] },
                        active: false
                    };

                        macro_to_add = &macros;

                        let mut add_keys: Vec<u32> = vec![];
                        let mut add_keys_converted: Vec<rdev::Key> = vec![];

                        match &macros.trigger {
                            TriggerEventType::KeyPressEvent { data: key } => {
                                for individual_keys in key{
                                    add_keys.push(individual_keys.keypress);
                                    add_keys_converted.push(SCANCODE_TO_RDEV[&individual_keys.keypress]);



                                }
                            }
                        }
                        output.push((add_keys.clone(), add_keys_converted.clone(), macro_to_add));


                }

            }

            println!("Macro Output Struct\n{:#?}", output);

            tuple
        }
    */
    pub fn read_data() -> MacroData {
        let path = "../data_json.json";

        let incoming_test: MacroData = MacroData {
            data: vec![Collection {
                name: "Default".to_string(),
                icon: 'i'.to_string(),
                macros: vec![],
                active: true,
            }],
        };

        let data = {
            match fs::read_to_string(path) {
                Ok(T) => T,
                Err(E) => {
                    println!("{}", E);
                    std::fs::write(
                        "../data_json.json",
                        serde_json::to_string_pretty(&incoming_test).unwrap(),
                    )
                        .unwrap();

                    let output = fs::read_to_string(path).unwrap();
                    println!("{}", output);

                    output
                }
            }
        };

        let deserialized: MacroData = serde_json::from_str(&data).unwrap();
        deserialized
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Collection {
    pub name: String,
    //TODO: base64 encoding
    pub icon: String,
    pub macros: Vec<Macro>,
    pub active: bool,
}

pub fn execute_macro(macros: &Macro) {
    for sequence in &macros.sequence {
        match sequence {
            ActionEventType::KeyPressEvent { data } => match data.keytype {
                KeyType::Down => send(&rdev::EventType::KeyPress(SCANCODE_TO_RDEV[&data.keypress])),
                KeyType::Up => send(&rdev::EventType::KeyRelease(
                    SCANCODE_TO_RDEV[&data.keypress],
                )),
                KeyType::DownUp => {
                    send(&rdev::EventType::KeyPress(SCANCODE_TO_RDEV[&data.keypress]));
                    thread::sleep(time::Duration::from_millis(*&data.press_duration as u64));
                    send(&rdev::EventType::KeyRelease(
                        SCANCODE_TO_RDEV[&data.keypress],
                    ));
                }
            },
            ActionEventType::PhillipsHueCommand { .. } => {}
            ActionEventType::OBS { .. } => {}
            ActionEventType::DiscordCommand { .. } => {}
            ActionEventType::UnicodeDirect { .. } => {}
            ActionEventType::Delay { data } => thread::sleep(time::Duration::from_millis(*data)),
        }
    }
}

///Main loop for now (of the library)
/// * `config` - &ApplicationConfig from the parsed JSON config file of the app.
pub fn run_this() {
    //==================================================
    //TODO: make this a grab instead of listen
    //TODO: try to make this interact better (cleanup the code a bit)
    //TODO: async the executor of the presses
    //TODO: io-uring async read files and write files
    //TODO: move all the plugins to its separate files (also with action keytype)

    loop {
        //Trigger hashes
        let trigger_overview = APPLICATION_STATE.data.read().unwrap().clone();

        //println!("{:#?}", trigger_overview);

        match APPLICATION_STATE.config.read().unwrap().use_input_grab {
            true => {
                let mut events = Vec::new();
                let mut pressed_keys: Vec<rdev::Key> = Vec::new();

                let (schan, rchan) = channel();
                let _grabber = thread::spawn(move || {
                    grab(move |event| match schan.send(event.clone()) {
                        Ok(T) => {
                            let mut keys_pressed: Vec<rdev::Key>;
                            match &event.event_type {
                                //TODO: Grab and discard the trigger actually
                                EventType::KeyPress(key) => Some(event),
                                _ => Some(event),
                            }
                        }
                        Err(_) => None,
                    })
                });

                for event in rchan.iter() {
                    events.push(event);

                    for i in &events {
                        //println!("{:?}", events.len());
                        match &i.event_type {
                            EventType::KeyPress(listened_key) => {
                                //TODO: Make this a hashtable or smth
                                pressed_keys.push(listened_key.clone());

                                for collections in &trigger_overview.data {
                                    if collections.active == true {
                                        for macros in &collections.macros {
                                            if macros.active == true {
                                                match &macros.trigger {
                                                    TriggerEventType::KeyPressEvent {
                                                        data: trigger_key,
                                                    } => {
                                                        let converted_keys: Vec<rdev::Key> =
                                                            trigger_key
                                                                .iter()
                                                                .map(|x| {
                                                                    SCANCODE_TO_RDEV[&x.keypress]
                                                                })
                                                                .collect();

                                                        if pressed_keys == converted_keys {
                                                            execute_macro(&macros);
                                                            println!("MACRO READY TO EXECUTE");
                                                        }
                                                        // if &events.first() == trigger_key.first(){
                                                        //
                                                        // }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }

                                println!("{:#?}", pressed_keys);
                                println!("Pressed: {:?}", listened_key);
                                //check_key(&pressed_keys);
                            }
                            EventType::KeyRelease(listened_key) => {
                                println!("Released: {:?}", listened_key);
                                pressed_keys.retain(|x| x != listened_key);
                                println!("{:#?}", pressed_keys);
                            }
                            EventType::ButtonPress(listened_key) => {
                                println!("MB Pressed:{:?}", listened_key)
                            }
                            EventType::ButtonRelease(listened_key) => {
                                println!("MB Released:{:?}", listened_key)
                            }
                            EventType::MouseMove { x, y } => (),
                            EventType::Wheel { delta_x, delta_y } => {}
                        }
                    }
                    events.pop();
                }
            }
            false => {
                let mut events = Vec::new();

                let (schan, rchan) = channel();
                let _listener = thread::spawn(move || {
                    listen(move |event| {
                        schan
                            .send(event)
                            .unwrap_or_else(|e| println!("Could not send event {:?}", e));
                    })
                        .expect("Could not listen");
                });

                for event in rchan.iter() {
                    events.push(event);

                    for i in &events {
                        //println!("{:?}", events.len());
                        match &i.event_type {
                            EventType::KeyPress(s) => {
                                println!("Pressed: {:?}", s);

                                //check_key(&s);
                            }
                            EventType::KeyRelease(s) => {
                                println!("Released: {:?}", s)
                            }
                            EventType::ButtonPress(s) => {
                                println!("MB Pressed:{:?}", s)
                            }
                            EventType::ButtonRelease(s) => {
                                println!("MB Released:{:?}", s)
                            }
                            EventType::MouseMove { x, y } => (),
                            EventType::Wheel { delta_x, delta_y } => {}
                        }
                    }
                    events.pop();
                }
            }
        }
    }
}

//TODO: Release the key actually?
fn send(event_type: &EventType) {
    let delay = time::Duration::from_millis(20);
    match simulate(event_type) {
        Ok(()) => (),
        Err(SimulateError) => {
            println!("We could not send {:?}", event_type);
        }
    }
    // Let ths OS catchup (at least MacOS)
    thread::sleep(delay);
}

fn get_user_input(display_text: String) -> String {
    println!("{}\n", display_text);

    let mut buffer: String = String::new();

    std::io::stdin()
        .read_line(&mut buffer)
        .expect("Invalid type");
    buffer.trim().to_string()
}
