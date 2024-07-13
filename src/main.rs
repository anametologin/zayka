use std::{
    future::pending,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use zbus::{connection, interface};

slint::include_modules!();

pub type Error = Box<dyn std::error::Error + Send + Sync>;

struct KeyPressed {
    action_dbus: Arc<Mutex<Action>>,
}

#[interface(name = "org.zay.KeyPressed1")]
impl KeyPressed {
    fn get_key_seq(&self) -> String {
        let empty = String::new();
        match &mut self.action_dbus.lock() {
            Ok(guard) => {
                if let Some(begin) = guard.begin {
                    let mut same_key: Option<&char> = None;
                    for key in &guard.keys {
                        if let Some(k) = same_key {
                            if *k != *key {
                                guard.begin = None;
                                guard.keys.clear();
                                return "#escape".to_string();
                            }
                        } else {
                            same_key = Some(key);
                        }
                    }
                    if (begin.elapsed() > guard.interval)
                        || (guard.keys.len() == 0)
                        || (guard.last_key_time.is_none())
                        || (guard.keys.len() > guard.keys_len as usize)
                        || ((guard.keys.len() < guard.keys_len as usize)
                            && if let Some(last_key_time) = guard.last_key_time {
                                last_key_time.elapsed() < Duration::from_millis(400)
                            } else {
                                false
                            })
                    {
                        return empty;
                    };
                    guard.begin = None;
                    let keys = guard.keys.clone();
                    guard.keys.clear();
                    for key in &keys {
                        if *key as u32 == 27 {
                            return "#escape".to_string();
                        }
                    }
                    return String::from_iter(keys);
                } else {
                    return empty;
                }
            }
            Err(e) => {
                println!("KeyPressed.begin.lock:{}", e);
                return "".to_string();
            }
        }
    }

    fn init_action(&mut self, dur: i32, keys_len: i32) {
        match &mut self.action_dbus.lock() {
            Ok(guard) => {
                if dur < 500 || dur > 10000 {
                    println!("duration can be more 500ms and less than 5000ms");
                    return;
                }
                if keys_len < 1 || keys_len > 255 {
                    println!("Keys length can be more zero and less than 255");
                    return;
                }

                guard.keys.clear();
                guard.keys_len = keys_len as u8;
                guard.begin = Some(Instant::now());
                guard.last_key_time = None;
                guard.interval = Duration::from_millis(dur as u64);
                // println!("Init action:{guard:?}");
            }
            Err(e) => {
                println!("KeyPressed.begin.lock:{}", e);
            }
        }
    }
}

#[derive(Debug)]
struct Action {
    keys: Vec<char>,
    keys_len: u8,
    begin: Option<Instant>,
    last_key_time: Option<Instant>,
    interval: Duration,
}
impl Action {
    fn new() -> Self {
        return Action {
            keys: Vec::new(),
            keys_len: 0,
            begin: None,
            last_key_time: None,
            interval: Duration::from_secs(0),
        };
    }
}
fn main() -> Result<(), slint::PlatformError> {
    let action: Arc<Mutex<Action>> = Arc::new(Mutex::new(Action::new()));
    let tokio_rt = tokio::runtime::Runtime::new().unwrap();

    let ui = AppWindow::new()?;
    ui.hide().unwrap();
    let action_dbus = Arc::clone(&action);
    slint::spawn_local(async move {
        let _ = tokio_rt.spawn(dbus_routine(action_dbus)).await.unwrap();
    })
    .unwrap();

    ui.on_key_press(move |key| {
        match &mut action.lock() {
            Ok(guard) => {
                if let Some(begin) = guard.begin {
                    if begin.elapsed() > guard.interval {
                        return;
                    };
                    let mut c: Vec<char> = key.chars().collect();
                    guard.keys.append(&mut c);
                    guard.last_key_time = Some(Instant::now());
                }
            }
            Err(e) => {
                println!("on_key_press.lock:{}", e);
            }
        }
        // println!("Key:{}", key);
    });

    ui.run()
}

async fn dbus_routine(action_dbus: Arc<Mutex<Action>>) -> Result<(), Error> {
    let key_pressed = KeyPressed { action_dbus };
    let _conn = connection::Builder::session()?
        .name("org.zay.KeyPressed")?
        .serve_at("/org/zay/KeyPressed", key_pressed)?
        .build()
        .await?;

    pending::<()>().await;
    Ok(())
}
