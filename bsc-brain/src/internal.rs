use crate::ApiError;


#[repr(u32)]
// These are set via external calls.
#[allow(dead_code)]
pub enum StatusCode {
    Ok = 0,
    HostError = 1,
    ArgumentError = 2,
    NotFound = 3,
}

impl StatusCode {
    pub fn to_result(self) -> Result<(), ApiError> {
        match self {
            StatusCode::Ok => Ok(()),
            StatusCode::HostError => Err(ApiError::HostError),
            StatusCode::ArgumentError => Err(ApiError::ArgumentError),
            StatusCode::NotFound => Err(ApiError::NotFound),
        }
    }

    pub fn to_num(self) -> u32 {
        self as u32
    }

    pub fn from_num(val: u32) -> Self {
        match val {
            0 => StatusCode::Ok,
            1 => StatusCode::HostError,
            2 => StatusCode::ArgumentError,
            3 => StatusCode::NotFound,
            _ => StatusCode::HostError,
        }
    }
}

pub type DroneID = u32;
