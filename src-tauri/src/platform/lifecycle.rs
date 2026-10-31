//! Native lock/suspend monitoring. No focus, tray, desktop indicator or notification.
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    thread::{self, JoinHandle},
    time::Duration,
};

pub struct LifecycleGuard {
    stop: Arc<AtomicBool>,
    worker: Option<JoinHandle<()>>,
}

impl LifecycleGuard {
    pub fn start(callback: impl Fn() + Send + Sync + 'static) -> Result<Self, &'static str> {
        Self::start_guarded(Arc::new(AtomicBool::new(false)), callback)
    }

    pub fn start_guarded(
        stop: Arc<AtomicBool>,
        callback: impl Fn() + Send + Sync + 'static,
    ) -> Result<Self, &'static str> {
        if stop.load(Ordering::Acquire) {
            return Err("Initialisation de la surveillance OS annulée.");
        }
        let invalidated = Arc::new(AtomicBool::new(false));
        let invalidated_callback = invalidated.clone();
        let callback = move || {
            // Latch even a lock followed by an unlock before startup completes.
            if !invalidated_callback.swap(true, Ordering::AcqRel) {
                callback();
            }
        };
        let worker_stop = stop.clone();
        let (ready_tx, ready_rx) = mpsc::sync_channel(1);
        let callback: Arc<dyn Fn() + Send + Sync> = Arc::new(callback);
        let worker = thread::spawn(move || {
            let result = monitor(worker_stop.clone(), callback.clone(), &ready_tx);
            if let Err(message) = result {
                let _ = ready_tx.try_send(Err(message));
                if !worker_stop.load(Ordering::Acquire) {
                    callback();
                }
            }
        });
        match ready_rx.recv_timeout(Duration::from_secs(8)) {
            Ok(Ok(())) if !stop.load(Ordering::Acquire) && !invalidated.load(Ordering::Acquire) => {
                Ok(Self {
                    stop,
                    worker: Some(worker),
                })
            }
            result => {
                stop.store(true, Ordering::Release);
                let _ = worker.join();
                Err(match result {
                    Ok(Err(message)) => message,
                    Ok(Ok(())) => "Initialisation de la surveillance OS invalidée.",
                    Err(_) => "Initialisation de la surveillance OS expirée.",
                })
            }
        }
    }
}

impl Drop for LifecycleGuard {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

#[cfg(target_os = "linux")]
fn monitor(
    stop: Arc<AtomicBool>,
    callback: Arc<dyn Fn() + Send + Sync>,
    ready: &mpsc::SyncSender<Result<(), &'static str>>,
) -> Result<(), &'static str> {
    use dbus::{
        blocking::{Connection, stdintf::org_freedesktop_dbus::Properties},
        message::MatchRule,
    };
    use std::time::Instant;
    let session = Connection::new_session()
        .map_err(|_| "Bus de session GNOME indisponible : vocal refusé.")?;
    let system =
        Connection::new_system().map_err(|_| "Bus système login1 indisponible : vocal refusé.")?;
    let screen = session.with_proxy(
        "org.gnome.ScreenSaver",
        "/org/gnome/ScreenSaver",
        Duration::from_secs(1),
    );
    let login = system.with_proxy(
        "org.freedesktop.login1",
        "/org/freedesktop/login1",
        Duration::from_secs(1),
    );
    let screen_callback = callback.clone();
    session
        .add_match(
            MatchRule::new_signal("org.gnome.ScreenSaver", "ActiveChanged")
                .with_sender("org.gnome.ScreenSaver")
                .with_path("/org/gnome/ScreenSaver"),
            move |(active,): (bool,), _, _| {
                if active {
                    screen_callback();
                }
                true
            },
        )
        .map_err(|_| "Abonnement verrouillage GNOME impossible.")?;
    let sleep_callback = callback.clone();
    system
        .add_match(
            MatchRule::new_signal("org.freedesktop.login1.Manager", "PrepareForSleep")
                .with_sender("org.freedesktop.login1")
                .with_path("/org/freedesktop/login1"),
            move |(sleeping,): (bool,), _, _| {
                if sleeping {
                    sleep_callback();
                }
                true
            },
        )
        .map_err(|_| "Abonnement veille login1 impossible.")?;
    // Subscribe before reading initial state, eliminating the initial-state race.
    let check = || -> Result<(), &'static str> {
        let (locked,): (bool,) = screen
            .method_call("org.gnome.ScreenSaver", "GetActive", ())
            .map_err(|_| "Surveillance GNOME perdue : vocal arrêté.")?;
        let sleeping: bool = login
            .get("org.freedesktop.login1.Manager", "PreparingForSleep")
            .map_err(|_| "Surveillance login1 perdue : vocal arrêté.")?;
        if locked || sleeping {
            return Err("Session verrouillée ou en veille : vocal refusé.");
        }
        Ok(())
    };
    // A daemon disappearance/replacement is not a benign unlock.
    for (connection, name) in [
        (&session, "org.gnome.ScreenSaver"),
        (&system, "org.freedesktop.login1"),
    ] {
        let cb = callback.clone();
        connection
            .add_match(
                MatchRule::new_signal("org.freedesktop.DBus", "NameOwnerChanged")
                    .with_sender("org.freedesktop.DBus"),
                move |(changed, old, new): (String, String, String), _, _| {
                    if changed == name && old != new {
                        cb();
                    }
                    true
                },
            )
            .map_err(|_| "Surveillance des services OS impossible.")?;
    }
    check()?;
    session
        .process(Duration::ZERO)
        .map_err(|_| "Surveillance GNOME perdue.")?;
    system
        .process(Duration::ZERO)
        .map_err(|_| "Surveillance login1 perdue.")?;
    ready.send(Ok(())).map_err(|_| "Initialisation annulée.")?;
    let mut health = Instant::now();
    while !stop.load(Ordering::Acquire) {
        session
            .process(Duration::from_millis(50))
            .map_err(|_| "Surveillance GNOME perdue.")?;
        system
            .process(Duration::from_millis(50))
            .map_err(|_| "Surveillance login1 perdue.")?;
        if health.elapsed() >= Duration::from_secs(1) {
            check()?;
            health = Instant::now();
        }
    }
    Ok(())
}

#[cfg(target_os = "windows")]
#[path = "lifecycle_windows.rs"]
mod windows;

#[cfg(target_os = "windows")]
use windows::monitor;

#[cfg(not(any(target_os = "linux", target_os = "windows")))]
fn monitor(
    _: Arc<AtomicBool>,
    _: Arc<dyn Fn() + Send + Sync>,
    _: &mpsc::SyncSender<Result<(), &'static str>>,
) -> Result<(), &'static str> {
    Err("Surveillance OS non implémentée sur cette cible ; manuel disponible.")
}
