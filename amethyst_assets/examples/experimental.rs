//! High level example
#![warn(rust_2018_idioms, rust_2018_compatibility)]

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use amethyst_assets::{Format, experimental::*};
use amethyst_core::ecs::{
    prelude::{Dispatcher, DispatcherBuilder, System, World, Write, WorldExt},
};
use amethyst_error::{format_err, Error, ResultExt};
use type_uuid::*;

#[derive(Debug, TypeUuid)]
#[uuid = "28d51c52-be81-4d99-8cdc-20b26eb12448"]
pub struct MeshAsset {
    // Just example fields
    buffer: Vec<[f32; 3]>,
    handle: GenericHandle,
    vec_handle_test: Vec<Handle<MeshAsset>>,
}

#[derive(Serialize, Deserialize, TypeUuid)]
#[uuid = "687b6d94-c653-4663-af73-e967c92ad140"]
pub struct VertexData {
    positions: Vec<[f32; 3]>,
    tex_coords: Vec<[f32; 2]>,
    handle: GenericHandle,
    vec_handle_test: Vec<Handle<MeshAsset>>,
}
// Registers the asset type which automatically prepares AssetStorage & ProcessingQueue
amethyst_assets::register_asset_type!(VertexData => MeshAsset; ProcessingSystem);
/// A format the mesh data could be stored with.
#[derive(Debug, Default, Clone, Serialize, Deserialize, TypeUuid)]
#[uuid = "df3c6c87-05e6-4cc9-8711-cb6a6aad9942"]
struct Ron;

impl Format<VertexData> for Ron {
    fn name(&self) -> &'static str {
        "RON"
    }

    fn import_simple(&self, bytes: Vec<u8>) -> Result<VertexData, Error> {
        let s = std::str::from_utf8(&bytes)?;
        ron::de::from_str(s).with_context(|e| format_err!("Failed to decode mesh file: {}", e))
    }
}
// Associates the .ron file extension with the Ron Format implementation
// The AssetDaemon will automatically trigger Ron import when a file is new/changed
amethyst_assets::register_importer!(".ron", Ron);

struct App {
    dispatcher: Dispatcher<'static, 'static>,
    state: Option<State>,
    world: World,
}

impl App {
    fn new(state: State) -> Self {
        let mut disp_builder = DispatcherBuilder::new();

        let mut world = World::new();

        let mut loader = DefaultLoader::default();
        loader.init_world(&mut world);
        loader.init_dispatcher(&mut disp_builder);
        world.insert(loader);

        App {
            dispatcher: disp_builder.build(),
            state: Some(state),
            world,
        }
    }

    fn update(&mut self) {
        self.dispatcher.dispatch(&mut self.world);
        self.world.maintain();
        let mut loader = self.world.write_resource::<DefaultLoader>();
        loader.process(&self.world).unwrap(); // TODO unwrap
    }

    fn run(&mut self) {
        loop {
            self.update();
            match self.state.take().unwrap().update(&mut self.world) {
                Some(state) => self.state = Some(state),
                None => return,
            }
            std::thread::sleep(std::time::Duration::from_millis(16));
        }
    }
}

#[derive(Default)]
pub struct ProcessingSystem;

impl<'a> System<'a> for ProcessingSystem {
    type SystemData = (
        Write<'a, ProcessingQueue<VertexData>>,
        Write<'a, AssetStorage<MeshAsset>>,
    );

    fn run(&mut self, (mut processing_queue, mut storage): Self::SystemData) {
        processing_queue.process(&mut *storage, |vertex_data| {
            Ok(ProcessingState::Loaded(MeshAsset { buffer: vertex_data.positions, handle: vertex_data.handle, vec_handle_test: vertex_data.vec_handle_test }))
        });
    }
}

enum State {
    Start,
    Loading(GenericHandle),
    SomethingElse(GenericHandle),
}

impl State {
    /// Returns `Some` if the app should quit.
    fn update(self, world: &mut World) -> Option<Self> {
        match self {
            State::Start => {
                let loader = world.read_resource::<DefaultLoader>();
                Some(State::Loading(
                    loader.load_asset_generic(
                        // TODO: implement a proc macro to parse asset uuids at compile time
                        // TODO: implement a generator for asset uuid constants based on asset daemon metadata
                        *uuid::Uuid::parse_str("39c7043a-dd7e-4654-9b22-e45d5c6b87cc")
                            .unwrap()
                            .as_bytes(),
                    ),
                ))
            }
            State::Loading(handle) => {
                // Check the load status - this could be a loading screen
                let loader = world.read_resource::<DefaultLoader>();
                match handle.load_status(&*loader) {
                    LoadStatus::Loaded => Some(State::SomethingElse(handle)),
                    _ => Some(State::Loading(handle)),
                }
            }
            State::SomethingElse(handle) => {
                // You could now start the actual game, cause the loading is done.
                // This example however will just quit.
                let storage = world.read_resource::<AssetStorage<MeshAsset>>();
                println!("Loaded asset {:?}", handle.asset_with_version(&storage));
                println!("Asset is loaded and the game can begin!");
                println!("Game ending, sorry");
                None
            }
        }
    }
}

fn main() {
    let examples_dir = PathBuf::from(format!("{}/examples", env!("CARGO_MANIFEST_DIR")));
    let assets_dir = examples_dir.join("assets");
    atelier_daemon::init_logging().expect("Failed to initialize logging");

    // launch an asset daemon in a separate thread
    std::thread::spawn(move || {
        atelier_daemon::AssetDaemon::default()
            .with_importers(atelier_importer::get_source_importers())
            .with_asset_dirs(vec![assets_dir])
            .with_db_path(examples_dir.join(".asset_db"))
            .run();
    });

    let mut app = App::new(State::Start);
    app.run();
}