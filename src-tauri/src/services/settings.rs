use crate::models::Setting;
use crate::state::ServiceContext;

pub fn get_setting(ctx: &ServiceContext, key: &str) -> Result<Option<String>, String> {
    ctx.db.get_setting(key).map_err(|e| e.to_string())
}

pub fn set_setting(ctx: &ServiceContext, key: &str, value: &str) -> Result<(), String> {
    ctx.db.set_setting(key, value).map_err(|e| e.to_string())
}

pub fn get_all_settings(ctx: &ServiceContext) -> Result<Vec<Setting>, String> {
    ctx.db.get_all_settings().map_err(|e| e.to_string())
        .map(|pairs| pairs.into_iter().map(|(k, v)| Setting { key: k, value: v }).collect())
}

pub fn delete_setting(ctx: &ServiceContext, key: &str) -> Result<(), String> {
    ctx.db.delete_setting(key).map_err(|e| e.to_string())
}
