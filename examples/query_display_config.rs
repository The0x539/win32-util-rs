fn main() {
    let dc = win32_util::display_config::query::active_paths().unwrap();
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
