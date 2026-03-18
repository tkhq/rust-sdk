use std::env;
use std::error::Error;
use turnkey_client::TurnkeyP256ApiKey;

// Convenience function shared across examples to load a Turnkey API key from the local `examples/.env` file, or from env vars.
pub fn load_api_key_from_env() -> Result<TurnkeyP256ApiKey, Box<dyn Error>> {
    // Load .env file from the example folder (examples/.env)
    let current_dir = env::current_dir()?; // should be the workspace root
    let env_path = current_dir.join("examples").join(".env");

    if env_path.exists() {
        dotenvy::from_path(&env_path)?;
    } else {
        println!("No .env file found at {env_path:?}");
        println!("Continuing because env might already be populated with the right variables");
    }

    let api_public_key =
        env::var("TURNKEY_API_PUBLIC_KEY").expect("cannot load TURNKEY_API_PUBLIC_KEY");
    let api_private_key =
        env::var("TURNKEY_API_PRIVATE_KEY").expect("cannot load TURNKEY_API_PRIVATE_KEY");

    Ok(TurnkeyP256ApiKey::from_strings(
        api_private_key,
        Some(api_public_key),
    )?)
}

// Convenience function to load the correct base URL. We allow overriding base URL with env vars or via TURNKEY_BASE_URL env vars
// This is useful in cases where you want to run examples against a local Turnkey stack, or dev/preprod.
// If nothing is specified or something errors, defaults to running against prod ("https://api.turnkey.com")
pub fn load_base_url_from_env() -> String {
    // Load .env file from the example folder (examples/.env)
    let current_dir =
        env::current_dir().expect("current_dir should succeed on a normal filesystem"); // should be the workspace root
    let env_path = current_dir.join("examples").join(".env");

    if env_path.exists() {
        dotenvy::from_path(&env_path).ok();
    } else {
        println!("No .env file found at {env_path:?}");
        println!("Continuing because env might already be populated with the right variables");
    }

    env::var("TURNKEY_API_BASE_URL").unwrap_or_else(|_| "https://api.turnkey.com".to_string())
}
