use anyhow::{anyhow, Result};
use clap::Parser;
use ksmap::synchronization::WorldSync;
use libks::{map_bin, world_ini};

use ksmap::{analysis, definitions};
use ksmap::drawing::{self, DrawOptions};
use ksmap::graphics::Graphics;
use ksmap::screen_map::ScreenMap;

mod cli;

fn main() -> Result<()> {
    let result = run();
    
    match &result {
        Ok(_) => println!("Success"),
        Err(err) => eprintln!("Error: {err}"),
    };

    result
}

pub fn run() -> Result<()> {
    let cli = cli::Cli::parse();

    let max_size = (cli.max_width / 600, cli.max_height / 240);
    if max_size.0 == 0 || max_size.1 == 0 {
        return Err(anyhow!("Maximum size was less than 1 screen"));
    }

    let level_dir =
        if cli.level.is_dir() {
            cli.level
        }
        else {
            cli.level
                .parent()
                .unwrap_or("".as_ref())
                .to_owned()
        };
    let data_dir = cli.data_dir.unwrap_or_else(|| level_dir.join("../../Data"));

    let ini = world_ini::load_ini_from_dir(&level_dir)?;
    
    let output_dir = cli.output_dir.unwrap_or_else(|| {
        let author = ini.get_in("World", "Author").unwrap_or("Unknown Author");
        let name = ini.get_in("World", "Name").unwrap_or("Unknown Title");

        format!("{author} - {name}").into()
    });

    if !output_dir.exists() {
        std::fs::create_dir(&output_dir)?;
    }

    let screens = map_bin::parse_map_file(level_dir.join("Map.bin"))?;
    let mut object_defs = definitions::load_object_defs("mapper_objects.toml")?;
    definitions::insert_custom_obj_defs(&mut object_defs, &ini);
    
    let mut gfx = Graphics::new(
        data_dir,
        &level_dir,
        &cli.templates_dir,
        &object_defs,
    );
    let assets_used = analysis::list_assets(&screens, &object_defs);
    
    print!("Loading assets...");
    gfx.load_tilesets(&assets_used.tilesets)?;
    gfx.load_gradients(&assets_used.gradients)?;
    gfx.load_objects(&assets_used.objects)?;
    println!(" Done");
    
    let screen_map = ScreenMap::new(screens);
    let strategy = cli.strategy.into_strategy(max_size);
    let partitions = strategy.partitions(&screen_map)?;

    println!("The level was partitioned into these regions:");
    for (i, partition) in partitions.iter().enumerate() {
        println!("    {}: {}", i + 1, partition.bounds())
    }
    println!();

    let options = DrawOptions {
        editor_only: cli.editor_only,
    };
    
    let world_sync = WorldSync::new(&screen_map, &object_defs);

    drawing::draw_partitions(&screen_map, &partitions, &gfx, &object_defs, &ini, output_dir, &options, &world_sync)?;

    Ok(())
}
