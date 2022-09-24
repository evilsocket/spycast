use crate::mdns::discovery::{MappedEndpoints, SharedEndpoints};

#[tauri::command]
fn get_state(state: tauri::State<'_, SharedEndpoints>) -> MappedEndpoints {
    state.lock().unwrap().clone()
}

pub fn run<T>(state: T)
where
    T: Send + Sync + 'static,
{
    tauri::Builder::default()
        .manage(state)
        .invoke_handler(tauri::generate_handler![get_state])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
