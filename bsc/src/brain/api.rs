use std::borrow::Cow;

use bsc_brain::{internal::StatusCode, *};
use wasmtime::Memory;
use zerocopy::{Immutable, IntoBytes};

use bevy::{ecs::query::QueryEntityError, prelude::*};

use super::runtime::{BrainHostApi, CallerParam};

pub enum ApiError {
    ModuleExternal(Cow<'static, str>),
    // An error that can be returned to the wrapped process
    ModuleInternal(StatusCode),
}

impl From<String> for ApiError {
    fn from(value: String) -> Self {
        Self::ModuleExternal(Cow::Owned(value))
    }
}

impl From<&'static str> for ApiError {
    fn from(value: &'static str) -> Self {
        Self::ModuleExternal(Cow::Borrowed(value))
    }
}

impl From<QueryEntityError> for ApiError {
    fn from(_value: QueryEntityError) -> Self {
        Self::ModuleInternal(StatusCode::NotFound)
    }
}

impl From<StatusCode> for ApiError {
    fn from(value: StatusCode) -> Self {
        Self::ModuleInternal(value)
    }
}

impl ApiError {
    fn unwrap_result(result: Result<(), ApiError>) -> StatusCode {
        match result {
            Ok(_) => StatusCode::Ok,
            Err(ApiError::ModuleExternal(err)) => {
                println!("Host error running module: {}", err);
                StatusCode::HostError
            }
            Err(ApiError::ModuleInternal(code)) => code,
        }
    }
}

pub type ApiResult<T = ()> = Result<T, ApiError>;

fn get_mem(c: &mut CallerParam) -> ApiResult<Memory> {
    Ok(match c.get_export("memory") {
        Some(wasmtime::Extern::Memory(mem)) => mem,
        _ => Err("Could not find wasm memory")?,
    })
}

fn write_mem<T: IntoBytes + Immutable>(c: &mut CallerParam, ptr: u32, value: T) -> ApiResult {
    get_mem(c)?
        .write(c, ptr as usize, value.as_bytes())
        .map_err(|_| StatusCode::ArgumentError)?;
    Ok(())
}

macro_rules! bsc_func {
    ($linker:ident, $method:ident) => {
        $linker.func_wrap("bsc_brain", stringify!($method), $method)
    };
}

fn drone_count(c: CallerParam) -> u32 {
    c.data().brain_ctx.get().drone_count()
}

fn drone_id(mut c: CallerParam, drone: u32, drone_id_ptr: u32) -> u32 {
    ApiError::unwrap_result((|| {
        let drone = c.data().brain_ctx.get().get_drone_at(drone as usize)?.id;

        write_mem(&mut c, drone_id_ptr, drone)?;
        Ok(())
    })())
    .to_num()
}

fn drone_status(mut c: CallerParam, drone: u32, status_ptr: u32) -> u32 {
    ApiError::unwrap_result((|| {
        let drone = c.data().brain_ctx.get().get_drone(drone)?;
        let status = DroneStatus { pos: drone.pos };

        write_mem(&mut c, status_ptr, status)?;
        Ok(())
    })())
    .to_num()
}

pub fn add_to_linker(l: &mut wasmtime::Linker<BrainHostApi>) -> Result<()> {
    bsc_func!(l, drone_count)?;
    bsc_func!(l, drone_id)?;
    bsc_func!(l, drone_status)?;
    Ok(())
}
