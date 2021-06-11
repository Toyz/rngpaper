#![windows_subsystem = "windows"]

mod structs;
extern crate native_windows_gui as nwg;
extern crate native_windows_derive as nwd;
#[macro_use] extern crate log;

use rand::seq::SliceRandom;
use wallpaper;
use log::LevelFilter;
use simplelog::{CombinedLogger, TermLogger, TerminalMode, Config, ColorChoice};
use nwd::NwgUi;
use nwg::NativeUi;
use std::thread;
use std::fs::{remove_file};

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
         thread::spawn(|| {
             let resp = load_all().unwrap();
             let item =  resp.choose(&mut rand::thread_rng()).unwrap();
             let prev_wallpaper = wallpaper::get().unwrap();

             info!("Current wallpaper: {}", prev_wallpaper);
             info!("Setting wallpaper to: {}", item.path);
             wallpaper::set_from_url(&item.path).unwrap();


             match remove_file(prev_wallpaper) {
                 Ok(_) => info!("Deleting previous wallpaper"),
                 Err(e) => error!("{}", e),
             }
         });
    }

    fn exit(&self) {
        nwg::stop_thread_dispatch();
    }

}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    CombinedLogger::init(
        vec![
            TermLogger::new(LevelFilter::Info, Config::default(), TerminalMode::Mixed, ColorChoice::Auto),
        ]
    )?;

    nwg::init()?;
    let _ui = SystemTray::build_ui(Default::default())?;
    nwg::dispatch_thread_events();

    Ok(())
}

fn load_all() -> Result<Vec<structs::Data>, Box<dyn std::error::Error>> {
    let resp = reqwest::blocking::get("https://wallhaven.cc/api/v1/search?q=@arkas&categories=010&purity=110")?
        .json::<structs::Root>()?;

    let mut items: Vec<structs::Data> = Vec::new();
    items = items.iter().chain(resp.data.iter()).cloned().collect();

    for n in 2..resp.meta.last_page {
        let resp = load_page(n)?;
        items = items.iter().chain(resp.iter()).cloned().collect();
    }

    Ok(items)
}

fn load_page(page: i64) -> Result<Vec<structs::Data>, Box<dyn std::error::Error>> {
    let url: String = format!("https://wallhaven.cc/api/v1/search?q=@arkas&categories=010&purity=110&page={}", page);

    let resp = reqwest::blocking::get(&url)?
        .json::<structs::Root>()?;

    Ok(resp.data)
}