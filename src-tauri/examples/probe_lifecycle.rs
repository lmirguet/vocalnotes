//! Host-only diagnostic: no microphone, keyring or provider contact.
fn main() {
    match vocalnotes_lib::platform::lifecycle::LifecycleGuard::start(|| {}) {
        Ok(guard) => {
            println!(
                "Surveillance OS initialisée ; ceci ne certifie pas les événements lock/suspend."
            );
            drop(guard);
        }
        Err(_) => {
            eprintln!(
                "Surveillance OS native indisponible : le produit doit refuser le vocal et garder le manuel."
            );
            std::process::exit(2);
        }
    }
}
