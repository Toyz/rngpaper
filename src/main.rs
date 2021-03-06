#![windows_subsystem = "windows"]

mod structs;
mod config;

extern crate native_windows_gui as nwg;
extern crate native_windows_derive as nwd;
extern crate dirs;
#[macro_use] extern crate log;

use rand::seq::SliceRandom;
use rand::Rng;
use wallpaper;
use log::LevelFilter;
use simplelog::{CombinedLogger, TermLogger, TerminalMode, Config, ColorChoice};
use nwd::NwgUi;
use nwg::NativeUi;
use std::thread;
use std::fs::{remove_file, create_dir_all, File,  remove_dir_all};
use std::thread::sleep;
use std::time::Duration;
use tauri_hotkey::{HotkeyManager, parse_hotkey};
use std::path::Path;
use reqwest::Url;
use std::borrow::Borrow;

#[derive(Default, NwgUi)]
pub struct SystemTray {
    #[nwg_control]
    window: nwg::MessageWindow,

    #[nwg_resource]
    embed: nwg::EmbedResource,

    #[nwg_resource(source_embed: Some(&data.embed), source_embed_str: Some("TRAY_ICON"))]
    icon: nwg::Icon,

    #[nwg_control(icon: Some(&data.icon), tip: Some("Hello"))]
    #[nwg_events(MousePressLeftUp: [SystemTray::show_menu], OnContextMenu: [SystemTray::show_menu])]
    tray: nwg::TrayNotification,

    #[nwg_control(parent: window, popup: true)]
    tray_menu: nwg::Menu,

    #[nwg_control(parent: tray_menu, text: "Change Wallpaper")]
    #[nwg_events(OnMenuItemSelected: [SystemTray::change_wallpaper])]
    tray_item1: nwg::MenuItem,

    #[nwg_control(parent: tray_menu, text: "Empty Background Cache")]
    #[nwg_events(OnMenuItemSelected: [SystemTray::empty_cache])]
    tray_item2: nwg::MenuItem,

    #[nwg_control(parent: tray_menu, text: "Exit")]
    #[nwg_events(OnMenuItemSelected: [SystemTray::exit])]
    tray_item3: nwg::MenuItem,
}
impl SystemTray {
    fn show_menu(&self) {
        let (x, y) = nwg::GlobalCursor::position();
        self.tray_menu.popup(x, y);
    }

    fn change_wallpaper(&self) {
        change_wallpaper()
    }

    fn empty_cache(&self) {
        let mut path = dirs::home_dir().ok_or("no path found").unwrap();
        path = path.join(".rngpaper").join("cache");

        remove_dir_all(path);
    }

    fn exit(&self) {
        nwg::stop_thread_dispatch();
    }
}

fn load_all() -> Result<Vec<structs::Data>, Box<dyn std::error::Error>> {
    let config = config::get_config();
    let config = config.lock().expect("Lock config failed");
    let collections = (*config).collections.as_ref().unwrap();
    debug!("{:?}", config);

    let api_key: String = match config.api_key.as_ref() {
        Some(key) => format!("&apikey={}", key),
        None => "".to_string()
    };

    let categories = config.categories.as_ref().unwrap();
    let purity = config.purity.as_ref().unwrap();

    let url = format!("https://wallhaven.cc/api/v1/search?q={}&categories={}&purity={}{}",
                      collections.choose(&mut rand::thread_rng()).unwrap(),
                      categories.to_string(),
                      purity.to_string(),
                      api_key
    );

    let resp = reqwest::blocking::get(&url)?
        .json::<structs::Root>()?;

    if resp.meta.last_page <= 0 {
        debug!("retrying in 250ms");
        sleep(Duration::from_millis(250));
        return load_all()
    }

    debug!("{:?}", resp);
    if resp.meta.last_page == 1 {
        return Ok(resp.data)
    }
    let page = rand::thread_rng().gen_range(1..resp.meta.last_page);
    if page == 1 {
        return Ok(resp.data)
    }

    let items = load_page(page, &url)?;

    Ok(items)
}

fn load_page(page: i64, base_url: &String) -> Result<Vec<structs::Data>, Box<dyn std::error::Error>> {
    let url: String = format!("{}&page={}", base_url, page);

    let resp = reqwest::blocking::get(&url)?
        .json::<structs::Root>()?;

    Ok(resp.data)
}

fn change_wallpaper() {
    thread::spawn(|| {
        let resp = match load_all() {
            Ok(data) => data,
            Err(e) => {
                error!("Error loading page {}", e);
                return
            }
        };
        let item = resp.choose(&mut rand::thread_rng()).unwrap();
        let prev_wallpaper = wallpaper::get().unwrap();

        info!("Current wallpaper: {}", prev_wallpaper);
        info!("Setting wallpaper to: {}", item.path);

        let mut path = dirs::home_dir().ok_or("no path found").unwrap();
        path = path.join(".rngpaper").join("cache");
        create_dir_all(&path);

        let url = Url::parse(item.path.as_str()).unwrap();
        let segments = url.path_segments().ok_or("no path segments").unwrap();
        let mut file_name = segments.last().ok_or("no file name").unwrap();
        if file_name.is_empty() {
            file_name = "wallpaper";
        }

        path = path.join(file_name);
        if path.exists() {
            wallpaper::set_from_path(path.to_str().unwrap()).unwrap();
            return;
        }

        let mut file = File::create(&path).unwrap();
        reqwest::blocking::get(item.path.as_str()).unwrap().copy_to(&mut file).unwrap();
        drop(file);

        info!("File path: {:?}", path);

        wallpaper::set_from_path(path.to_str().unwrap()).unwrap();
    /*
        match remove_file(prev_wallpaper) {
            Ok(_) => info!("Deleting previous wallpaper"),
            Err(e) => error!("{}", e),
        }
     */
    });
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // TODO: make this configurable
    CombinedLogger::init(
        vec![
            TermLogger::new(LevelFilter::Debug, Config::default(), TerminalMode::Mixed, ColorChoice::Auto),
        ]
    )?;

    let config = config::get_config();
    let config = config.lock().expect("Lock config failed");

    let mut hkm = HotkeyManager::default();
    hkm.register(parse_hotkey(config.hotkey.as_ref().unwrap())?, change_wallpaper);
    std::mem::drop(config);

    info!("Running?");
    nwg::init()?;
    let _ui = SystemTray::build_ui(Default::default())?;
    nwg::dispatch_thread_events();

    Ok(())
}