use std::sync::Mutex;

use bevy::prelude::*;
use bsc_brain::DroneStatus;
use wasmtime::Caller;
use wasmtime_wasi::{WasiCtxBuilder, preview1::WasiP1Ctx};

#[derive(Resource)]
struct BrainEngine {
    engine: wasmtime::Engine,
    linker: wasmtime::Linker<BrainHostApi>,
}

impl Default for BrainEngine {
    fn default() -> Self {
        let engine = wasmtime::Engine::default();
        let mut linker = wasmtime::Linker::new(&engine);

        wasmtime_wasi::preview1::add_to_linker_sync::<BrainHostApi>(&mut linker, |t| {
            t.wasi_ctx
                .get_mut()
                .expect("Failed to acquire wasi ctx lock")
        })
        .expect("Failed to add wasi methods.");

        register_brain_funcs(&mut linker).unwrap();
        Self { engine, linker }
    }
}

type CallerParam<'a> = Caller<'a, BrainHostApi>;

fn _get_wasm_ptr<'a, T: Sized>(c: &mut CallerParam<'a>, ptr: u32) -> anyhow::Result<Option<&'a T>> {
    let mem = match c.get_export("memory") {
        Some(wasmtime::Extern::Memory(mem)) => mem,
        _ => anyhow::bail!("Could not find wasm memory"),
    };
    let ptr = ptr as usize;
    let output: Option<&'a T> = mem
        .data(c)
        .get(ptr..(ptr + size_of::<T>()))
        .map(|arr| unsafe { &*(arr.as_ptr() as *const T) });

    Ok(output)
}

fn get_wasm_ptr_mut<'a, T: Sized>(
    c: &mut CallerParam<'a>,
    ptr: u32,
) -> anyhow::Result<Option<&'a mut T>> {
    let mem = match c.get_export("memory") {
        Some(wasmtime::Extern::Memory(mem)) => mem,
        _ => anyhow::bail!("Could not find wasm memory"),
    };
    let ptr = ptr as usize;
    let output: Option<&'a mut T> = mem
        .data_mut(c)
        .get(ptr..(ptr + size_of::<T>()))
        .map(|arr| unsafe { &mut *(arr.as_ptr() as *mut T) });

    Ok(output)
}

const BSC_MODULE: &'static str = "bsc_brain";

fn register_brain_funcs(l: &mut wasmtime::Linker<BrainHostApi>) -> anyhow::Result<()> {
    l.func_wrap(BSC_MODULE, "drone_count", || -> u32 { 2 })?;
    l.func_wrap(BSC_MODULE, "drone_id", |_index: u32| -> u32 { 2 })?;
    l.func_wrap(BSC_MODULE, "drone_ping", |_drone: u32| -> u32 { 2 })?;
    l.func_wrap(
        BSC_MODULE,
        "drone_status",
        |mut c: CallerParam, _drone: u32, status_ptr: u32| -> i32 {
            if let Ok(Some(status)) = get_wasm_ptr_mut::<DroneStatus>(&mut c, status_ptr) {
                *status = DroneStatus {
                    pos: [1., 2., 3., 4., 5.],
                };
            }
            0
        },
    )?;
    Ok(())
}

fn _to_api_mut(api_wrapper: &mut Option<BrainHostApi>) -> &mut BrainHostApi {
    api_wrapper
        .as_mut()
        .expect("Host API was not set during wasm callback.")
}
fn _to_api(api_wrapper: &Option<BrainHostApi>) -> &BrainHostApi {
    api_wrapper
        .as_ref()
        .expect("Host API was not set during wasm callback.")
}

// Contains references to the queries that fulfill the api.
struct BrainHostApi {
    wasi_ctx: Mutex<WasiP1Ctx>,
}

#[derive(Component)]
struct Brain {
    status: BrainStatus,
    _wall_clock_time: std::time::Duration,
    _drones: Vec<Entity>,
}

enum BrainStatus {
    Running {
        store: wasmtime::Store<BrainHostApi>,
        update: wasmtime::TypedFunc<(), ()>,
        _shutdown: wasmtime::TypedFunc<(), ()>,
    },
    Crashed {
        _msg: String,
    },
}

impl Brain {
    fn new(engine: &BrainEngine, module: &wasmtime::Module) -> Self {
        let status = (|| {
            let wasi_ctx = WasiCtxBuilder::new().inherit_stdio().build_p1();
            let mut store = wasmtime::Store::new(
                &engine.engine,
                BrainHostApi {
                    wasi_ctx: Mutex::new(wasi_ctx),
                },
            );

            let instance = engine.linker.instantiate(&mut store, module)?;

            let init = instance.get_typed_func::<(), ()>(&mut store, "brain_init")?;
            let update = instance.get_typed_func::<(), ()>(&mut store, "brain_update")?;
            let shutdown = instance.get_typed_func::<(), ()>(&mut store, "brain_shutdown")?;

            init.call(&mut store, ())?;

            Ok(BrainStatus::Running {
                store,
                update,
                _shutdown: shutdown,
            })
        })()
        .unwrap_or_else(|e: anyhow::Error| {
            println!("Error starting brain: {:?}", e);
            BrainStatus::Crashed {
                _msg: e.to_string(),
            }
        });

        Brain {
            status,
            _wall_clock_time: default(),
            _drones: default(),
        }
    }

    fn run(&mut self) {
        if let BrainStatus::Running { store, update, .. } = &mut self.status {
            //*store.data_mut() = Some(unsafe { transmute(api) });

            update
                .call(&mut *store, ())
                .expect("Error while running wasm brain.");

            //*store.data_mut() = None;
        }
    }
}

#[derive(Component)]
struct Drone {}

fn run_drones(mut brains: Query<&mut Brain>, _drone_pos: Query<(&Drone, &mut Transform)>) {
    for mut brain in brains.iter_mut() {
        brain.run();
    }
}

fn spawn_entities(mut c: Commands, mut engine: ResMut<BrainEngine>) {
    let wat = include_bytes!("../test_brain.wasm");
    let module = wasmtime::Module::new(&engine.engine, wat).expect("Wasm module did not compile.");

    c.spawn(Brain::new(&mut engine, &module));

    c.spawn((Drone {}, Transform::default()));
}

fn main() -> anyhow::Result<()> {
    App::default()
        .add_plugins(DefaultPlugins)
        .init_resource::<BrainEngine>()
        .add_systems(Startup, spawn_entities)
        .add_systems(Update, run_drones)
        .run();
    Ok(())
}
