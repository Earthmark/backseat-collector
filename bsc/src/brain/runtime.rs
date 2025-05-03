use bevy::prelude::*;
use bsc_brain::internal::StatusCode;
use wasmtime::Caller;
use wasmtime_wasi::{WasiCtxBuilder, preview1::WasiP1Ctx};

use crate::drone::Drone;

use super::{
    BrainDroneLinks, BrainStats,
    api::{self, ApiResult},
};

#[derive(Resource)]
pub struct BrainEngine {
    engine: wasmtime::Engine,
    linker: wasmtime::Linker<BrainHostApi>,
}

impl Default for BrainEngine {
    fn default() -> Self {
        let engine = wasmtime::Engine::default();
        let mut linker = wasmtime::Linker::new(&engine);

        wasmtime_wasi::preview1::add_to_linker_sync::<BrainHostApi>(&mut linker, |t| {
            &mut t.wasi_ctx
        })
        .expect("Failed to add wasi methods.");

        api::add_to_linker(&mut linker).unwrap();
        Self { engine, linker }
    }
}

pub type CallerParam<'a> = Caller<'a, BrainHostApi>;

// Contains references to the queries that fulfill the api.
pub struct BrainHostApi {
    wasi_ctx: WasiP1Ctx,
    pub brain_ctx: BrainCtxHolder,
}

#[derive(Default)]
pub struct BrainCtxHolder(Option<&'static mut BrainCtx<'static, 'static, 'static, 'static>>);

impl BrainCtxHolder {
    fn set(&mut self, ctx: &mut BrainCtx) {
        self.0 = Some(unsafe { std::mem::transmute(ctx) });
    }

    fn clear(&mut self) {
        self.0 = None
    }

    pub fn get<'a>(&'a self) -> &'a BrainCtx<'a, 'a, 'a, 'a> {
        let r = self
            .0
            .as_ref()
            .expect("Brain ctx not set while wasm was running");
        unsafe {
            std::mem::transmute::<
                &BrainCtx<'static, 'static, 'static, 'static>,
                &'a BrainCtx<'a, 'a, 'a, 'a>,
            >(*r)
        }
    }

    pub fn _get_mut<'a>(&'a mut self) -> &'a mut BrainCtx<'a, 'a, 'a, 'a> {
        let r = self
            .0
            .as_mut()
            .expect("Brain ctx not set while wasm was running");
        unsafe {
            std::mem::transmute::<
                &mut BrainCtx<'static, 'static, 'static, 'static>,
                &'a mut BrainCtx<'a, 'a, 'a, 'a>,
            >(*r)
        }
    }
}

pub struct BrainCtx<'world, 'state, 'arg, 'ctx> {
    _stats: &'ctx mut BrainStats,
    brain_links: Option<&'state BrainDroneLinks>,
    drone_pos_query: Query<'world, 'state, &'arg Drone>,
}

impl<'world, 'state, 'arg, 'ctx> BrainCtx<'world, 'state, 'arg, 'ctx> {
    pub fn new(
        stats: &'ctx mut BrainStats,
        brain_links: Option<&'state BrainDroneLinks>,
        drone_pos_query: Query<'world, 'state, &'arg Drone>,
    ) -> Self {
        Self {
            _stats: stats,
            brain_links,
            drone_pos_query,
        }
    }

    pub fn drone_count(&self) -> u32 {
        self.drone_pos_query.iter().count() as u32
    }

    pub fn get_drone(&self, drone_id: u32) -> ApiResult<&Drone> {
        if let Some(links) = self.brain_links {
            self.drone_pos_query
                .iter_many(&links.0)
                .find(|d| d.id == drone_id)
                .ok_or(api::ApiError::ModuleInternal(StatusCode::NotFound))
        } else {
            Err(api::ApiError::ModuleInternal(StatusCode::NotFound))
        }
    }

    pub fn get_drone_at(&self, drone_index: usize) -> ApiResult<&Drone> {
        if let Some(links) = self.brain_links {
            if let Some(drone_entity) = links.0.get(drone_index) {
                return Ok(self.drone_pos_query.get(*drone_entity)?);
            }
        }
        Err(api::ApiError::ModuleInternal(StatusCode::NotFound))
    }
}

pub struct BrainRuntime {
    store: wasmtime::Store<BrainHostApi>,
    update: wasmtime::TypedFunc<(), ()>,
    _shutdown: wasmtime::TypedFunc<(), ()>,
}

impl BrainRuntime {
    pub fn new(engine: &BrainEngine, module: impl AsRef<[u8]>) -> Result<Self> {
        let wasi_ctx = WasiCtxBuilder::new().inherit_stdio().build_p1();
        let mut store = wasmtime::Store::new(
            &engine.engine,
            BrainHostApi {
                wasi_ctx,
                brain_ctx: default(),
            },
        );

        let module = wasmtime::Module::new(&engine.engine, module)?;

        let instance = engine.linker.instantiate(&mut store, &module)?;

        let init = instance.get_typed_func::<(), ()>(&mut store, "brain_init")?;
        let update = instance.get_typed_func::<(), ()>(&mut store, "brain_update")?;
        let shutdown = instance.get_typed_func::<(), ()>(&mut store, "brain_shutdown")?;

        init.call(&mut store, ())?;

        Ok(BrainRuntime {
            store,
            update,
            _shutdown: shutdown,
        })
    }

    pub fn run(&mut self, ctx: &mut BrainCtx) {
        self.store.data_mut().brain_ctx.set(ctx);

        self.update
            .call(&mut self.store, ())
            .expect("Error while running wasm brain.");

        self.store.data_mut().brain_ctx.clear();
    }
}
