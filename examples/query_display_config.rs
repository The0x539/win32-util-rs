use win32_util::display_config::{DisplayConfig, ModeInfo, QueryFlags};

fn main() {
    let dc = DisplayConfig::active_paths(QueryFlags::all()).unwrap();
    for path in dc.paths {
        println!("\t- flags: {:?}", path.flags);

        let source = &path.source;
        print!(
            "\t  source {:?}: modes[{}], {:?}",
            source.id, source.source_mode_idx, source.status_flags
        );
        if let Some(id) = source.clone_group_id {
            print!(", clone group {id}");
        }
        println!();

        let target = &path.target;
        println!(
            "\t  target {:?}: modes[{}], {:?}",
            target.id, target.target_mode_idx, target.status_flags
        );
        println!("\t\toutput_technology: {:?}", target.output_technology);
        println!("\t\trotation: {:?}", target.rotation);
        println!("\t\tscaling: {:?}", target.scaling);
        println!("\t\trefresh_rate: {:?}", target.refresh_rate);
        println!("\t\tscanline_ordering: {:?}", target.scanline_ordering);
        println!("\t\tavailable: {:?}", target.target_available);
        if let Some(idx) = target.desktop_image_idx {
            println!("\t\tdesktop_image: modes[{idx}]");
        }
    }
    println!();
    println!("modes:");
    for (i, mode) in dc.modes.iter().enumerate() {
        match mode.mode {
            ModeInfo::Target(target) => {
                println!("\t[{i}] target {:?}:", mode.id);
                println!("\t\tpixel_rate: {}", target.pixel_rate);
                println!("\t\thsync: {}", target.h_sync_freq);
                println!("\t\tvsync: {}", target.v_sync_freq);
                println!("\t\tactive_size: {:?}", target.active_size);
                println!("\t\ttotal_size: {:?}", target.total_size);
                println!("\t\tvideo_standard: {:?}", target.video_standard);
                println!("\t\tvsync_freq_divider: {:?}", target.vsync_freq_divider);
                println!("\t\tscanline_ordering: {:?}", target.scanline_ordering);
            }
            ModeInfo::Source(source) => println!(
                "\t[{i}] source {:?}: {:?}, {:?}",
                mode.id, source.location, source.pixel_format
            ),
            ModeInfo::DesktopImage(image) => {
                println!("\t[{i}] desktop image {:?}:", mode.id);
                println!("\t\tsource_size: {:?}", image.path_source_size);
                println!("\t\tregion: {:?}", image.region);
                println!("\t\tclip: {:?}", image.clip);
            }
        }
    }
    println!();
}
