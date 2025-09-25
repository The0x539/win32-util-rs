use win32_util::display_config::{
    DisplayConfig, QueryFlags,
    set::{ApplyFlags, ValidateFlags},
};

fn main() {
    let mut dc = DisplayConfig::active_paths(QueryFlags::all()).unwrap();

    // Assuming the PC has three 1440p monitors arranged horizontally,
    // with the primary display in the center, set the left and right
    // displays to operate in clone mode.
    for path in &mut dc.paths {
        let pos = path.source.mode(&dc.modes).location.top_left();
        if pos == (-2560, 0) || pos == (2560, 0) {
            path.set_clone_group(1);
        }
    }

    dc.validate(ValidateFlags::VIRTUAL_MODE).unwrap();
    dc.apply(ApplyFlags::VIRTUAL_MODE).unwrap();
}
