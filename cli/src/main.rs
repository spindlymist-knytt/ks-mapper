use anyhow::{anyhow, Result};
use clap::Parser;
use libks::{map_bin, world_ini};

use ks_render_map::definitions;
use ks_render_map::drawing::{self, DrawOptions};
use ks_render_map::graphics::GraphicsLoader;
use ks_render_map::screen_map::ScreenMap;

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

    let mut object_defs = definitions::load_object_defs("mapper_objects.toml")?;
    definitions::insert_custom_obj_defs(&mut object_defs, &ini);
    
    let mut gfx = GraphicsLoader::new(
        data_dir,
        &level_dir,
        &cli.templates_dir,
        object_defs,
    );

    let screens = {
        let screens = map_bin::parse_map_file(level_dir.join("Map.bin"))?;
        ScreenMap::new(screens)
    };

    let strategy = cli.strategy.into_strategy(max_size);
    let partitions = strategy.partitions(&screens)?;

    println!("The level was partitioned into these regions:");
    for (i, partition) in partitions.iter().enumerate() {
        println!("    {}: {}", i + 1, partition.bounds())
    }
    println!();

    let options = DrawOptions {
        editor_only: cli.editor_only,
    };

    drawing::draw_partitions(&screens, &partitions, &mut gfx, &ini, output_dir, &options)?;

    Ok(())
}
