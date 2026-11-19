use clap::builder::StyledStr;

#[derive(Debug)]
pub enum UuidRequest {
    Dissect {},
    Info(StyledStr),
    V1 {},
    V2 {},
    V3 {},
    V4 {},
    V5 {},
    V6 {},
    V7 {},
    V8 {},
    Validate {},
}

#[derive(Debug)]
pub enum UuidResponse {
    Dissect {},
    Info(StyledStr),
    V1 {},
    V2 {},
    V3 {},
    V4 {},
    V5 {},
    V6 {},
    V7 {},
    V8 {},
    Validate {},
}
