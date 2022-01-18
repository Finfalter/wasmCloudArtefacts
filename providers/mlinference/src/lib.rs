use wasmcloud_interface_mlinference::{ ResultStatus };

pub type BindlePath = String;
pub type ModelName = String;

pub fn get_valid_status() -> ResultStatus {
    ResultStatus {
        has_error: false,
        error: None
    }
}