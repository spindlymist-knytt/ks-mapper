use std::io::Write;

use anyhow::{anyhow, Result};
use clap::Parser;
use ksmap::partition::{GridPartitioner, IslandsPartitioner, Partitioner};
use ksmap::seed::MapSeed;
use ksmap::synchronization::{SyncOptions, WorldSync};
use ksmap::timespan::Timespan;
use libks::{map_bin, world_ini};

use ksmap::{analysis, definitions};
use ksmap::drawing::{self, DrawOptions};
use ksmap::graphics::Graphics;
use ksmap::screen_map::ScreenMap;

use crate::cli::PartitionStrategy;

mod cli;

fn main() -> Result<()> {
    let mut span_total = Timespan::begin();
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

    let seed = match cli.seed.map(|seed_str| MapSeed::try_from(seed_str)) {
        Some(Ok(seed)) => seed,
        Some(Err(err)) => {
            eprintln!("Failed to parse seed. The seed must be 1-16 hex digits (0-9 A-F).");
            return Err(err.into());
        },
        None => MapSeed::random(),
    };
    println!("Seed: {seed}");

    print!("Loading map");
    let _ = std::io::stdout().flush();
    let mut span_map = Timespan::begin();
    let screens = map_bin::parse_map_file(level_dir.join("Map.bin"))?;
    span_map.end();
    println!(" [{span_map}]");
    
    print!("Loading definitions");
    let _ = std::io::stdout().flush();
    let mut span_defs = Timespan::begin();
    let mut object_defs = definitions::load_object_defs(cli.object_definitions)?;
    definitions::insert_custom_obj_defs(&mut object_defs, &ini);
    span_defs.end();
    println!(" [{span_defs}]");
    
    let mut gfx = Graphics::new(
        data_dir,
        &level_dir,
        &cli.templates_dir,
        &object_defs,
    );
    
    print!("Loading assets");
    let _ = std::io::stdout().flush();
    let mut span_assets = Timespan::begin();
    let assets_used = analysis::list_assets(&screens, &object_defs);
    gfx.load_tilesets(&assets_used.tilesets)?;
    gfx.load_gradients(&assets_used.gradients)?;
    gfx.load_objects(&assets_used.objects)?;
    span_assets.end();
    println!(" [{span_assets}]");

    let screen_map = ScreenMap::new(screens);
    
    print!("Synchronizing map");
    let _ = std::io::stdout().flush();
    let mut span_sync = Timespan::begin();
    let sync_options = SyncOptions {
        maximize_visible_lasers: !cli.randomize_lasers,
    };
    let world_sync = WorldSync::new(seed, &screen_map, &object_defs, &sync_options);
    span_sync.end();
    println!(" [{span_sync}]");
    
    let strategy: Box<dyn Partitioner> = match cli.partitioner {
        PartitionStrategy::Islands => Box::new(IslandsPartitioner {
            max_size,
            gap: cli.islands_args.min_gap..=cli.islands_args.max_gap,
        }),
        PartitionStrategy::Grid => Box::new(GridPartitioner {
            max_size,
            rows: cli.grid_args.rows,
            cols: cli.grid_args.cols,
        }),
    };
    
    print!("Partitioning:");
    let _ = std::io::stdout().flush();
    let mut span_partitions = Timespan::begin();
    let partitions = strategy.partitions(&screen_map);
    span_partitions.end();
    println!(" [{span_partitions}]");
    for (i, partition) in partitions.iter().enumerate() {
        println!("    {}: {}", i + 1, partition.bounds())
    }
    println!();

    let draw_options = DrawOptions {
        editor_only: cli.editor_only,
        use_multithreaded_encoder: !cli.single_threaded_encoder,
    };
    drawing::draw_partitions(seed, &screen_map, &partitions, &gfx, &object_defs, &ini, output_dir, &draw_options, &world_sync)?;
    
    span_total.end();
    println!("Finished in {span_total}");

    Ok(())
}
