mod paths;

use std::{collections::BTreeMap, env, fs, path::{Path, PathBuf}};

use anyhow::{Result, bail};
use clap::{Parser, Subcommand, Args};
use ksmap::{
    analysis,
    definitions,
    drawing::{self, DrawContext, DrawOptions, export_canvas_multithreaded},
    graphics::Graphics,
    partition::{GridPartitioner, Partitioner},
    screen_map::ScreenMap,
    seed::MapSeed,
    synchronization::{SyncOptions, WorldSync},
};
use libks::{map_bin, world_ini};
use serde::{Deserialize, Serialize};

use paths::*;

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    task: Task,
}

#[derive(Subcommand, Clone)]
enum Task {
    MakeSeeds(MakeSeedsArgs)
}

#[derive(Args, Clone)]
struct MakeSeedsArgs {
    #[arg(short, default_value = "3")]
    n: usize,
    #[arg(default_value = "*")]
    glob: String,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.task {
        Task::MakeSeeds(args) => make_seeds(args),
    }
}

#[derive(Serialize, Deserialize)]
struct SeedIndexEntry {
    seeds: Vec<MapSeed>,
}

fn make_seeds(args: MakeSeedsArgs) -> Result<()> {
    if args.glob.contains(['/', '\\']) {
        bail!("Glob pattern should not contain a slash");
    }
    let glob = glob::glob(&args.glob)?;
    
    let level_names = {
        let current_dir = env::current_dir()?;
        env::set_current_dir(WORLDS_DIR.as_path())?;
        
        let mut level_names = Vec::<String>::new();
        for path in glob {
            let path = path?;
            if path.is_dir()
                && let Some(level_name) = path.to_str()
            {
                level_names.push(level_name.to_owned());
            }
        }
        
        env::set_current_dir(current_dir)?;
        level_names
    };
    
    let mut seed_index: BTreeMap<String, SeedIndexEntry> = {
        if SEED_INDEX_PATH.exists() {
            let contents = std::fs::read_to_string(SEED_INDEX_PATH.as_path())?;
            toml::from_str(&contents)?
        }
        else {
            BTreeMap::new()
        }
    };
    
    for level_name in level_names {
        let level_dir = WORLDS_DIR.join(&level_name);
        let output_dir = SEEDS_DIR.join(&level_name);
        let seeds: Vec<_> = (0..args.n).map(|_| MapSeed::random()).collect();
        
        if output_dir.exists() {
            std::fs::remove_dir_all(&output_dir)?;
        }
        std::fs::create_dir_all(&output_dir)?;
        render_seeds(&level_dir, &seeds, &output_dir);
        
        seed_index.insert(level_name, SeedIndexEntry {
            seeds,
        });
    }
    
    let seed_index_serialized = toml::to_string_pretty(&seed_index)?;
    fs::write(SEED_INDEX_PATH.as_path(), seed_index_serialized)?;
    
    Ok(())
}

fn render_seeds(level_dir: &Path, seeds: &[MapSeed], output_dir: &Path) {
    let ini = world_ini::load_ini_from_dir(&level_dir)
        .expect("World.ini should be valid");
    let screens = map_bin::parse_map_file(level_dir.join("Map.bin"))
        .expect("Map.bin should be valid");
    
    let mut object_defs = definitions::load_object_defs(DEFINITIONS_PATH.as_path())
        .expect("Object definitions should be valid");
    definitions::insert_custom_obj_defs(&mut object_defs, &ini);
    
    let mut gfx = Graphics::new(
        DATA_DIR.as_path(),
        &level_dir,
        TEMPLATES_DIR.as_path(),
        &object_defs,
    );
    let assets_used = analysis::list_assets(&screens, &object_defs);
    
    gfx.load_tilesets(&assets_used.tilesets)
        .expect("IO error or corrupt image while loading tilesets");
    gfx.load_gradients(&assets_used.gradients)
        .expect("IO error or corrupt image while loading gradients");
    gfx.load_objects(&assets_used.objects)
        .expect("IO error or corrupt image while loading objects");
    
    let screen_map = ScreenMap::new(screens);
    
    let strategy = GridPartitioner::default();
    let partitions = strategy.partitions(&screen_map);
    assert!(partitions.len() == 1);
    let partition = &partitions[0];
    
    let draw_options = DrawOptions {
        editor_only: false,
    };
    let sync_options = SyncOptions {
        maximize_visible_lasers: true,
    };
    
    for seed in seeds.iter().cloned() {
        let world_sync = WorldSync::new(seed, &screen_map, &object_defs, &sync_options);
        
        let draw_context = DrawContext {
            seed,
            screens: &screen_map,
            gfx: &gfx,
            defs: &object_defs,
            ini: &ini,
            world_sync: &world_sync,
            options: draw_options,
        };
        
        let canvas = drawing::draw_partition(draw_context, partition)
            .expect("IO error while drawing map");
        
        let output_path = output_dir.join(format!("{seed}.png"));
        export_canvas_multithreaded(canvas, &output_path)
            .expect("Error while exporting map");
    }
}
