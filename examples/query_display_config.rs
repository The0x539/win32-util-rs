use win32_util::display_config::DisplayConfig;

fn main() {
    let dc = DisplayConfig::active_paths().unwrap();
    println!("paths:");
    for path in dc.paths {
        println!("\t- source: {:?}", path.source);
        println!("\t  target: {:?}", path.target);
    }
    println!();
    println!("modes:");
    for mode in dc.modes {
        println!("\t{mode:?}");
    }
    println!();
}
