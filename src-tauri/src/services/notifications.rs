use crate::models::NotificationSetting;
use crate::state::ServiceContext;

pub fn get_notification_setting(ctx: &ServiceContext, target_id: &str, target_type: &str) -> Result<Option<String>, String> {
    ctx.db.get_notification_setting(target_id, target_type).map_err(|e| e.to_string())
}

pub fn set_notification_setting(ctx: &ServiceContext, target_id: &str, target_type: &str, level: &str) -> Result<(), String> {
    ctx.db.set_notification_setting(target_id, target_type, level).map_err(|e| e.to_string())
}

pub fn get_all_notification_settings(ctx: &ServiceContext) -> Result<Vec<NotificationSetting>, String> {
    ctx.db.get_all_notification_settings().map_err(|e| e.to_string())
}
