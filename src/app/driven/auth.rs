use ring::rand::SecureRandom;
use uuid::Uuid;

fn generate_challenge() -> Vec<u8> {
    let rng = ring::rand::SystemRandom::new();
    let mut challenge = vec![0u8; 32];
    rng.fill(&mut challenge)
        .expect("Failed to generate random challenge");
    challenge
}

fn generate_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs()
}

pub struct StoredChallenge {
    challenge: Vec<u8>,
    timestamp: u64,
    user_id: Uuid,
}

pub struct ChallangeStore {
    challenges: std::collections::HashMap<Uuid, StoredChallenge>,
}


