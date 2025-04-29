use std::env;
use std::error::Error;
use std::time;
use tkhq_api_key_stamper::TurnkeyApiKey;

pub fn current_time_ms() -> u128 {
    time::SystemTime::now()
        .duration_since(time::UNIX_EPOCH)
        .unwrap()
        .as_millis()
}

// Convenience function shared across examples to load a Turnkey API key from the local `examples/.env` file, or from env vars.
pub fn load_api_key_from_env() -> Result<TurnkeyApiKey, Box<dyn Error>> {
    // Load .env file from the example folder (examples/.env)
    let current_dir = env::current_dir()?; // should be the workspace root
    let env_path = current_dir.join("examples").join(".env");

    if env_path.exists() {
        dotenvy::from_path(&env_path)?;
    } else {
        println!("No .env file found at {:?}", env_path);
        println!("Continuing because env might already be populated with the right variables");
    }

    let api_public_key =
        env::var("TURNKEY_API_PUBLIC_KEY").expect("cannot load TURNKEY_API_PUBLIC_KEY");
    let api_private_key =
        env::var("TURNKEY_API_PRIVATE_KEY").expect("cannot load TURNKEY_API_PRIVATE_KEY");

    Ok(TurnkeyApiKey {
        private_key_hex: api_private_key,
        public_key_hex: api_public_key.clone(),
    })
}
